//! Template command - Manage debate templates

use crate::templates::TemplateManager;

/// List all available templates
pub async fn list_templates() -> anyhow::Result<()> {
    let manager = TemplateManager::new()?;
    let templates = manager.list_templates();

    if templates.is_empty() {
        println!("No templates available.");
        return Ok(());
    }

    println!("Available Templates:");
    println!();

    for template in templates {
        let source = if template.is_builtin {
            "built-in"
        } else {
            "user"
        };

        println!("  {} ({})", template.name, source);
        println!("      {}", template.description);
        println!(
            "      Participants: {}, Rounds: {}",
            template.participant_count, template.default_rounds
        );
        println!();
    }

    println!("Use a template: gptengage debate \"topic\" --template <name>");

    Ok(())
}

/// Show template details
pub async fn show_template(name: String) -> anyhow::Result<()> {
    let manager = TemplateManager::new()?;

    match manager.get_template(&name) {
        Some(template) => {
            println!("Template: {}", template.name);
            println!("Description: {}", template.description);
            println!("Default Rounds: {}", template.default_rounds);
            println!();

            println!("Participants:");
            for (i, p) in template.participants.iter().enumerate() {
                println!();
                println!("  {}. {} ({})", i + 1, p.persona, p.cli);
                println!("     Instructions: {}", p.instructions);
                if !p.expertise.is_empty() {
                    println!("     Expertise: {}", p.expertise.join(", "));
                }
            }

            if let Some(ref ctx) = template.context {
                println!();
                println!("Context:");
                if let Some(ref prefix) = ctx.prefix {
                    println!("  Prefix: {}", prefix);
                }
                if let Some(ref suffix) = ctx.suffix {
                    println!("  Suffix: {}", suffix);
                }
            }

            println!();
            println!("Usage: gptengage debate \"<topic>\" --template {}", name);

            Ok(())
        }
        None => Err(anyhow::anyhow!(
            "Template '{}' not found. Use 'gptengage template list' to see available templates.",
            name
        )),
    }
}
