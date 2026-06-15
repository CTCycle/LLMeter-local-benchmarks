use std::process;

use clap::Parser;
use colored::Colorize;

use llmeter::cli::{self, Cli};
use llmeter::config::{self, AppConfig};
use llmeter::errors::LLMeterError;
use llmeter::progress::TerminalProgressRenderer;
use llmeter::providers::ProviderClient;

fn main() {
    let cli = Cli::parse();
    let result = run(cli);

    match result {
        Ok(code) => process::exit(code),
        Err(err) => {
            if err.downcast_ref::<LLMeterError>().is_some() {
                eprintln!("{} {}", "Error:".red().bold(), err);
                process::exit(2);
            }
            eprintln!("{} {}", "Error:".red().bold(), err);
            process::exit(1);
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    let config = AppConfig::from_env(&cli);

    match cli.command {
        None | Some(cli::Commands::Menu) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout);
            llmeter::ui::main_menu(&config, &client)?;
            Ok(0)
        }
        Some(cli::Commands::Status) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout);
            llmeter::ui::print_status_panel(&client.status());
            Ok(0)
        }
        Some(cli::Commands::Providers {
            ref provider_command,
        }) => {
            match provider_command {
                cli::ProviderCommands::List => llmeter::ui::print_provider_catalog(),
                cli::ProviderCommands::Set { provider } => {
                    let path = config::save_global_provider(*provider)?;
                    println!(
                        "Saved default provider '{}' to {}",
                        provider,
                        path.display()
                    );
                }
            }
            Ok(0)
        }
        Some(cli::Commands::Models { json }) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout);
            let models = client.list_models()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&models)?);
            } else {
                llmeter::ui::print_models(&models, config.provider);
            }
            Ok(0)
        }
        Some(cli::Commands::Show { ref model }) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout);
            let info = client.show_model(model)?;
            println!("{}", serde_json::to_string_pretty(&info)?);
            Ok(0)
        }
        Some(cli::Commands::Bench { ref bench_command }) => {
            match bench_command {
                cli::BenchCommands::Menu => {
                    let client =
                        ProviderClient::new(config.provider, &config.base_url, config.timeout);
                    llmeter::ui::benchmark_menu(&config, &client)?;
                }
                cli::BenchCommands::List { suite } => {
                    llmeter::ui::print_benchmark_catalog(*suite);
                }
                cli::BenchCommands::Run {
                    provider,
                    suite,
                    ref models,
                    ref benchmarks,
                    runs,
                    max_tokens,
                    temperature,
                    ref export,
                    ref report,
                    ref param,
                } => {
                    let run_config = provider
                        .map(|selected| config.with_provider(selected))
                        .unwrap_or_else(|| config.clone());
                    let client = ProviderClient::new(
                        run_config.provider,
                        &run_config.base_url,
                        run_config.timeout,
                    );
                    let available_models = llmeter::runner::installed_model_names(&client)?;
                    let selected_models: Vec<String> = match models.as_deref() {
                        None => return Err(anyhow::anyhow!("--models is required for non-interactive benchmark runs. Use 'all' or a comma-separated list.")),
                        Some(m) if m.trim().to_lowercase() == "all" => available_models,
                        Some(m) => m.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                    };

                    let (selected_benchmarks, all_benchmarks) = match benchmarks.as_deref() {
                        None | Some("all") => (None, true),
                        Some(b) => (
                            Some(
                                b.split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect::<Vec<_>>(),
                            ),
                            false,
                        ),
                    };

                    let extra_params = cli::parse_params(param)?;
                    let mut progress = TerminalProgressRenderer::new();
                    let run = llmeter::runner::run_benchmarks(
                        &client,
                        &run_config,
                        llmeter::runner::BenchmarkRunRequest {
                            suite: *suite,
                            model_names: &selected_models,
                            benchmark_ids: selected_benchmarks.as_deref(),
                            all_benchmarks,
                            runs: runs.unwrap_or(run_config.default_runs),
                            max_tokens: max_tokens.unwrap_or(run_config.default_max_tokens),
                            temperature: temperature.unwrap_or(run_config.default_temperature),
                            extra_options: &extra_params,
                        },
                        2,
                        Some(&mut progress),
                    )?;

                    let saved = llmeter::runner::save_outputs(
                        &run_config,
                        &run,
                        export,
                        report,
                        Some(&mut progress),
                    )?;
                    llmeter::ui::summarize_run(&run);
                    llmeter::ui::print_saved_paths(&saved);
                }
                cli::BenchCommands::Perf {
                    provider,
                    ref models,
                    profile,
                    ref prompt_tokens,
                    ref output_tokens,
                    ref concurrency,
                    warmup,
                    runs,
                    stream,
                    no_stream,
                    ref jsonl,
                    ref export,
                    ref report,
                    ref param,
                } => {
                    let run_config = provider
                        .map(|selected| config.with_provider(selected))
                        .unwrap_or_else(|| config.clone());
                    let client = ProviderClient::new(
                        run_config.provider,
                        &run_config.base_url,
                        run_config.timeout,
                    );
                    let available_models = llmeter::runner::installed_model_names(&client)?;
                    let selected_models: Vec<String> = match models.as_deref() {
                        None => return Err(anyhow::anyhow!("--models is required for performance runs. Use 'all' or a comma-separated list.")),
                        Some(m) if m.trim().to_lowercase() == "all" => available_models,
                        Some(m) => m.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                    };
                    let extra_params = cli::parse_params(param)?;
                    let plan = llmeter::performance::config::PerformancePlan::from_cli(
                        run_config.provider,
                        selected_models,
                        *profile,
                        prompt_tokens.as_deref(),
                        output_tokens.as_deref(),
                        concurrency.as_deref(),
                        *warmup,
                        *runs,
                        !no_stream || *stream,
                        jsonl.clone(),
                        extra_params,
                    )?;
                    let mut progress = TerminalProgressRenderer::new();
                    let run = llmeter::performance::runner::run_performance_plan(
                        &run_config,
                        &client,
                        plan,
                        Some(&mut progress),
                    )?;
                    let saved = llmeter::runner::save_outputs(
                        &run_config,
                        &run,
                        export,
                        report,
                        Some(&mut progress),
                    )?;
                    llmeter::ui::summarize_run(&run);
                    llmeter::ui::print_saved_paths(&saved);
                }
            }
            Ok(0)
        }
        Some(cli::Commands::Report { ref report_command }) => {
            llmeter::runner::command_report(&config, report_command)?;
            Ok(0)
        }
        Some(cli::Commands::Quality {
            ref quality_command,
        }) => {
            match quality_command {
                cli::QualityCommands::List => llmeter::ui::print_quality_catalog(),
                cli::QualityCommands::Plan {
                    framework,
                    task,
                    model,
                } => {
                    let plan =
                        llmeter::quality::adapter::build_quality_plan(*framework, task, model);
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                }
            }
            Ok(0)
        }
        Some(cli::Commands::Help { ref topic }) => {
            llmeter::ui::print_help_topic(topic.as_deref());
            Ok(0)
        }
    }
}
