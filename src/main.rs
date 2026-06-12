use std::process;

use clap::Parser;
use colored::Colorize;

use llmeter::cli::{self, Cli};
use llmeter::config::AppConfig;
use llmeter::errors::LLMeterError;
use llmeter::ollama::client::OllamaClient;
use llmeter::ollama::server::OllamaServerManager;

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
    let client = OllamaClient::new(&config.api_base_url, config.timeout);
    let manager = OllamaServerManager::new(client.clone(), config.state_dir.clone());

    match cli.command {
        None | Some(cli::Commands::Menu) => {
            llmeter::ui::main_menu(&config, &client, &manager)?;
            Ok(0)
        }
        Some(cli::Commands::Status) => {
            llmeter::ui::print_status_panel(&manager.status(), &manager.installed_version_cli());
            Ok(0)
        }
        Some(cli::Commands::Server { ref server_command }) => {
            match server_command {
                cli::ServerCommands::Status => {
                    llmeter::ui::print_status_panel(&manager.status(), &manager.installed_version_cli());
                }
                cli::ServerCommands::Start => {
                    let status = manager.start(10.0)?;
                    llmeter::ui::print_status_panel(&status, &manager.installed_version_cli());
                }
                cli::ServerCommands::Stop { force } => {
                    let msg = manager.stop(*force)?;
                    println!("{msg}");
                }
            }
            Ok(0)
        }
        Some(cli::Commands::Models { json }) => {
            let models = client.list_models()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&models)?);
            } else {
                llmeter::ui::print_models(&models);
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
                    llmeter::ui::benchmark_menu(&config, &client, &manager)?;
                }
                cli::BenchCommands::List => {
                    llmeter::ui::print_benchmark_catalog();
                }
                cli::BenchCommands::Run {
                    ref models,
                    ref benchmarks,
                    runs,
                    num_predict,
                    temperature,
                    ref export,
                    ref report,
                    start_server,
                    ref option,
                } => {
                    if *start_server && !client.is_running() {
                        manager.start(10.0)?;
                    }

                    let available_models = llmeter::runner::installed_model_names(&client)?;
                    let selected_models: Vec<String> = match models.as_deref() {
                        None => return Err(anyhow::anyhow!("--models is required for non-interactive benchmark runs. Use 'all' or a comma-separated list.")),
                        Some(m) if m.trim().to_lowercase() == "all" => available_models,
                        Some(m) => m.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                    };

                    let (selected_benchmarks, all_benchmarks) = match benchmarks.as_deref() {
                        None | Some("all") => (None, true),
                        Some(b) => (Some(b.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>()), false),
                    };

                    let run = llmeter::runner::run_benchmarks(
                        &client,
                        &config,
                        &selected_models,
                        selected_benchmarks.as_deref(),
                        all_benchmarks,
                        runs.unwrap_or(config.default_runs),
                        num_predict.unwrap_or(config.default_num_predict),
                        temperature.unwrap_or(config.default_temperature),
                        &cli::parse_options(option)?,
                    )?;

                    llmeter::ui::summarize_run(&run);
                    let saved = llmeter::runner::save_outputs(&config, &run, export, report)?;
                    llmeter::ui::print_saved_paths(&saved);
                }
            }
            Ok(0)
        }
        Some(cli::Commands::Report { ref report_command }) => {
            llmeter::runner::command_report(&config, report_command)?;
            Ok(0)
        }
    }
}
