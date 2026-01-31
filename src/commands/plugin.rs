//! Plugin command - Manage CLI plugins

use crate::invokers::base::command_exists;
use crate::plugins::PluginManager;

/// List all installed plugins
pub async fn list_plugins() -> anyhow::Result<()> {
    let manager = PluginManager::new()?;
    let plugins = manager.list_plugins();

    if plugins.is_empty() {
        println!("No plugins installed.");
        println!();
        println!("To install a plugin, create a TOML file in ~/.gptengage/plugins/");
        println!("Example: ~/.gptengage/plugins/ollama.toml");
        return Ok(());
    }

    println!("Installed Plugins:");
    println!();

    for plugin in plugins {
        let available = command_exists(&plugin.detection.check_command);
        let status = if available { "✓" } else { "✗" };

        println!(
            "  {} {} ({})",
            status, plugin.plugin.name, plugin.plugin.description
        );
        println!("      Command: {}", plugin.plugin.command);
        println!("      Prompt mode: {:?}", plugin.invoke.prompt_mode);
        if !available {
            println!(
                "      Warning: '{}' not found in PATH",
                plugin.detection.check_command
            );
        }
        println!();
    }

    Ok(())
}

/// Validate a plugin file without installing
pub async fn validate_plugin(path: String) -> anyhow::Result<()> {
    match PluginManager::validate_plugin_file(&path) {
        Ok(config) => {
            println!("✓ Plugin file is valid");
            println!();
            println!("Plugin Details:");
            println!("  Name: {}", config.plugin.name);
            println!("  Description: {}", config.plugin.description);
            println!("  Command: {}", config.plugin.command);
            println!("  Prompt mode: {:?}", config.invoke.prompt_mode);

            // Check if the command is available
            let available = command_exists(&config.detection.check_command);
            if available {
                println!("  Status: ✓ CLI available");
            } else {
                println!(
                    "  Status: ✗ CLI not found ({})",
                    config.detection.check_command
                );
            }

            println!();
            println!(
                "To install, copy the file to ~/.gptengage/plugins/{}.toml",
                config.plugin.name
            );

            Ok(())
        }
        Err(e) => {
            println!("✗ Plugin file is invalid");
            println!();
            println!("Error: {}", e);
            Err(e)
        }
    }
}
