mod cli;
mod function;
mod runtime;
mod trigger;

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};
use std::sync::Arc;

use cli::{Cli, Commands};
use function::FunctionRegistry;
use runtime::WasmRuntime;
use trigger::{SimpleTrigger, Trigger as _};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let runtime = WasmRuntime::new();
    let registry = Arc::new(FunctionRegistry::new(runtime));
    let trigger = SimpleTrigger::new(registry.clone());

    match cli.command {
        Commands::Deploy {
            name,
            file,
            trigger: subjects,
        } => {
            let id = registry.register_function(&name, &file, subjects)?;
            println!("Function '{name}' deployed with id: {id}");
        }

        Commands::List => {
            let functions = registry.list_functions();
            if functions.is_empty() {
                println!("No functions deployed.");
                return Ok(());
            }

            println!("List of deployed functions:");
            println!("{:<10} {:<20} {:<20}", "ID", "Name", "Triggers");
            println!("{:-<60}", "");

            for func in functions {
                println!(
                    "{:<10} {:<20} {:<20}",
                    func.id,
                    func.name,
                    func.trigger_subjects.join(", ")
                );
            }
        }

        Commands::Invoke { id, subject, data } => {
            println!("Invoking function '{id}' with '{subject}' trigger...");
            let result = registry.invoke_function(&id, &subject, data.as_bytes().to_vec())?;

            match result {
                Some(output) => match String::from_utf8(output.clone()) {
                    Ok(text) => println!("Response: {}", text),
                    Err(_) => println!("Response: {:?} (binary)", output),
                },
                None => println!("No response"),
            }
        }

        Commands::Start => {
            println!("Starting WASM Lambda server...");
            println!("To simulate function invocation, enter the following format:");
            println!("trigger_subject payload");
            println!("To exit, enter 'exit'");

            trigger.start().await?;

            loop {
                print!("> ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if input == "exit" {
                    break;
                }

                let parts: Vec<&str> = input.splitn(2, ' ').collect();
                if parts.len() < 2 {
                    println!("Invalid format. Enter in the format 'trigger_subject payload'.");
                    continue;
                }

                let subject = parts[0];
                let payload = parts[1].as_bytes().to_vec();

                match trigger.trigger(subject, payload).await {
                    Ok(results) => {
                        if results.is_empty() {
                            println!("No functions connected to '{subject}' trigger.");
                        } else {
                            for (i, result) in results.iter().enumerate() {
                                match result {
                                    Some(output) => match String::from_utf8(output.clone()) {
                                        Ok(text) => println!("Function {}: {}", i, text),
                                        Err(_) => println!("Function {}: {:?} (binary)", i, output),
                                    },
                                    None => println!("Function {}: No response", i),
                                }
                            }
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }

            trigger.stop().await?;
            println!("Server stopped");
        }
    }

    Ok(())
}
