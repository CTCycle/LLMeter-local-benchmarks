use std::io::{self, IsTerminal};
use std::process;

use clap::{CommandFactory, Parser};
use colored::Colorize;

use llmeter::cli::{self, Cli};
use llmeter::config::{self, AppConfig};
use llmeter::errors::LLMeterError;
use llmeter::lifecycle;
use llmeter::progress::TerminalProgressRenderer;
use llmeter::providers::ProviderClient;

fn main() {
    let cli = Cli::parse();
    let result = run(cli);

    match result {
        Ok(code) => process::exit(code),
        Err(err) => {
            if err.downcast_ref::<LLMeterError>() == Some(&LLMeterError::Interrupted) {
                process::exit(130);
            }
            if err.downcast_ref::<LLMeterError>() == Some(&LLMeterError::Canceled) {
                process::exit(0);
            }
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
    let terminal_capable = io::stdin().is_terminal() && io::stdout().is_terminal()
        || cfg!(windows) && std::env::var_os("LLMETER_CONPTY").is_some();
    if matches!(cli.command, None | Some(cli::Commands::Menu)) && !terminal_capable {
        Cli::command().print_help()?;
        println!();
        return Ok(2);
    }
    let config = AppConfig::from_env(&cli)?;

    match cli.command {
        None | Some(cli::Commands::Menu) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout)?;
            llmeter::ui::main_menu(&config, &client)?;
            Ok(if llmeter::ui::take_interrupt_requested() {
                130
            } else {
                0
            })
        }
        Some(cli::Commands::Status) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout)?;
            let status = client.status();
            let running = status.running;
            llmeter::ui::print_status_panel(&status);
            Ok(if running { 0 } else { 1 })
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
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout)?;
            let models = client.list_models_fresh()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&models)?);
            } else {
                llmeter::ui::print_models(&models, config.provider);
            }
            Ok(0)
        }
        Some(cli::Commands::Show { ref model }) => {
            let client = ProviderClient::new(config.provider, &config.base_url, config.timeout)?;
            let info = client.show_model(model)?;
            println!("{}", serde_json::to_string_pretty(&info)?);
            Ok(0)
        }
        Some(cli::Commands::Bench { ref bench_command }) => {
            match bench_command {
                cli::BenchCommands::Menu => {
                    let client =
                        ProviderClient::new(config.provider, &config.base_url, config.timeout)?;
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
                    include_response_preview,
                } => {
                    let run_config = match provider {
                        Some(selected) => config.with_provider(*selected)?,
                        None => config.clone(),
                    };
                    let client = ProviderClient::new(
                        run_config.provider,
                        &run_config.base_url,
                        run_config.timeout,
                    )?;
                    let available_models = llmeter::runner::installed_model_names(&client)?;
                    let selected_models: Vec<String> = match models.as_deref() {
                        None => {
                            return Err(anyhow::anyhow!(
                                "--models is required for non-interactive benchmark runs. Use 'all' or a comma-separated list."
                            ));
                        }
                        Some(m) if m.trim().to_lowercase() == "all" => available_models,
                        Some(m) => m
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
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
                        export.as_str(),
                        report.as_str(),
                        llmeter::results::OutputPrivacyPolicy {
                            include_response_preview: *include_response_preview,
                            redact_sensitive_values: true,
                        },
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
                    no_stream,
                    allow_large_prompt,
                    allow_large_matrix,
                    ref jsonl,
                    ref export,
                    ref report,
                    ref param,
                    include_response_preview,
                    load_measurement,
                    load_probe_runs,
                    telemetry,
                    sample_interval_ms,
                    ref provider_process,
                    probe_capabilities,
                    probe_all_endpoints,
                    ref model_cache_dir,
                    scan_model_cache,
                    detail,
                    dry_run,
                    max_requests,
                } => {
                    let run_config = match provider {
                        Some(selected) => config.with_provider(*selected)?,
                        None => config.clone(),
                    };
                    let client = ProviderClient::new(
                        run_config.provider,
                        &run_config.base_url,
                        run_config.timeout,
                    )?;
                    let available_models = llmeter::runner::installed_model_names(&client)?;
                    let selected_models: Vec<String> = match models.as_deref() {
                        None => {
                            return Err(anyhow::anyhow!(
                                "--models is required for performance runs. Use 'all' or a comma-separated list."
                            ));
                        }
                        Some(m) if m.trim().to_lowercase() == "all" => available_models,
                        Some(m) => m
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    };
                    let extra_params = cli::parse_params(param)?;
                    let stream_enabled = !*no_stream;
                    let plan = llmeter::performance::config::PerformancePlan::from_cli_with_safety(
                        run_config.provider,
                        selected_models,
                        *profile,
                        prompt_tokens.as_deref(),
                        output_tokens.as_deref(),
                        concurrency.as_deref(),
                        *warmup,
                        *runs,
                        stream_enabled,
                        jsonl.clone(),
                        extra_params,
                        llmeter::performance::config::PerformanceSafetyOptions {
                            allow_large_prompt: *allow_large_prompt,
                            allow_large_matrix: *allow_large_matrix,
                        },
                        *load_measurement,
                        *load_probe_runs,
                        *telemetry,
                        *sample_interval_ms,
                        provider_process.clone(),
                        *probe_capabilities,
                        *probe_all_endpoints,
                        model_cache_dir.clone(),
                        *scan_model_cache,
                        *detail,
                        Some(*max_requests),
                    )?;
                    print_performance_estimate(&plan, *max_requests);
                    if *dry_run {
                        return Ok(0);
                    }
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
                        export.as_str(),
                        report.as_str(),
                        llmeter::results::OutputPrivacyPolicy {
                            include_response_preview: *include_response_preview,
                            redact_sensitive_values: true,
                        },
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
                    let plan = llmeter::quality::adapter::build_quality_plan(
                        *framework,
                        task,
                        model,
                        &config.base_url,
                    );
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                }
            }
            Ok(0)
        }
        Some(cli::Commands::Install { ref bin_dir, force }) => {
            let result = lifecycle::install(bin_dir.as_deref(), force)?;
            println!("{}", result.summary);
            for detail in result.details {
                println!("  - {detail}");
            }
            Ok(0)
        }
        Some(cli::Commands::Update {
            ref source,
            ref bin_dir,
        }) => {
            let result = lifecycle::update(bin_dir.as_deref(), source.as_deref())?;
            println!("{}", result.summary);
            for detail in result.details {
                println!("  - {detail}");
            }
            Ok(0)
        }
        Some(cli::Commands::Uninstall {
            ref bin_dir,
            purge_home,
        }) => {
            let result = lifecycle::uninstall(bin_dir.as_deref(), purge_home)?;
            println!("{}", result.summary);
            for detail in result.details {
                println!("  - {detail}");
            }
            Ok(0)
        }
        Some(cli::Commands::Help { ref topic }) => {
            llmeter::ui::print_help_topic(topic.as_deref());
            Ok(0)
        }
    }
}

fn print_performance_estimate(
    plan: &llmeter::performance::config::PerformancePlan,
    max_requests: u32,
) {
    println!("Performance plan estimate:");
    println!("  Models: {}", plan.models.join(", "));
    println!(
        "  Prompt tokens: {}",
        plan.prompt_sizes
            .estimated_tokens
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "  Output tokens: {}",
        plan.output_sizes
            .estimated_tokens
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "  Concurrency: {}",
        plan.concurrency
            .levels
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("  Scenarios: {}", plan.scenario_count());
    println!("  Warmup requests: {}", plan.total_warmup_requests());
    println!("  Measured requests: {}", plan.total_measured_requests());
    println!("  Total requests: {}", plan.total_requests());
    println!("  Max requests: {max_requests}");
}
