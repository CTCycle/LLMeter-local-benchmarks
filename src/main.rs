use std::process;

use clap::Parser;
use colored::Colorize;

use llmeter::cli::{self, Cli};
use llmeter::config::AppConfig;
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
    let client = ProviderClient::new(config.provider, &config.base_url, config.timeout);

    match cli.command {
        None | Some(cli::Commands::Menu) => {
            llmeter::ui::main_menu(&config, &client)?;
            Ok(0)
        }
        Some(cli::Commands::Status) => {
            llmeter::ui::print_status_panel(&client.status());
            Ok(0)
        }
        Some(cli::Commands::Providers {
            ref provider_command,
        }) => {
            match provider_command {
                cli::ProviderCommands::List => llmeter::ui::print_provider_catalog(),
            }
            Ok(0)
        }
        Some(cli::Commands::Models { json }) => {
            let models = client.list_models()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&models)?);
            } else {
                llmeter::ui::print_models(&models, config.provider);
            }
            Ok(0)
        }
        Some(cli::Commands::Show { ref model }) => {
            let info = client.show_model(model)?;
            println!("{}", serde_json::to_string_pretty(&info)?);
            Ok(0)
        }
        Some(cli::Commands::Bench { ref bench_command }) => {
            match bench_command {
                cli::BenchCommands::Menu => {
                    llmeter::ui::benchmark_menu(&config, &client)?;
                }
                cli::BenchCommands::List => {
                    llmeter::ui::print_benchmark_catalog();
                }
                cli::BenchCommands::Run {
                    ref models,
                    ref benchmarks,
                    runs,
                    max_tokens,
                    temperature,
                    ref export,
                    ref report,
                    ref param,
                } => {
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
                        &config,
                        llmeter::runner::BenchmarkRunRequest {
                            model_names: &selected_models,
                            benchmark_ids: selected_benchmarks.as_deref(),
                            all_benchmarks,
                            runs: runs.unwrap_or(config.default_runs),
                            max_tokens: max_tokens.unwrap_or(config.default_max_tokens),
                            temperature: temperature.unwrap_or(config.default_temperature),
                            extra_options: &extra_params,
                        },
                        2,
                        Some(&mut progress),
                    )?;

                    let saved = llmeter::runner::save_outputs(
                        &config,
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
        Some(cli::Commands::Help { ref topic }) => {
            llmeter::ui::print_help_topic(topic.as_deref());
            Ok(0)
        }
    }
}
