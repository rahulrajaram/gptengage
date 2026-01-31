//! Built-in debate templates
//!
//! These templates are compiled into the binary and available without
//! any additional configuration.

use super::{DebateTemplate, TemplateContext, TemplateParticipant};
use std::collections::HashMap;

/// Get all built-in templates
pub fn get_builtin_templates() -> HashMap<String, DebateTemplate> {
    let mut templates = HashMap::new();

    templates.insert("code-review".to_string(), code_review_template());
    templates.insert(
        "architecture-decision".to_string(),
        architecture_decision_template(),
    );
    templates.insert("security-audit".to_string(), security_audit_template());
    templates.insert("api-design".to_string(), api_design_template());
    templates.insert(
        "incident-postmortem".to_string(),
        incident_postmortem_template(),
    );

    templates
}

fn code_review_template() -> DebateTemplate {
    DebateTemplate {
        name: "code-review".to_string(),
        description: "Multi-perspective code review with security, performance, and maintainability focus".to_string(),
        default_rounds: 2,
        participants: vec![
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Security Reviewer".to_string(),
                instructions: "Focus on security vulnerabilities, input validation, authentication issues, and OWASP Top 10 concerns. Flag any potential injection attacks, insecure data handling, or authentication bypasses.".to_string(),
                expertise: vec![
                    "security".to_string(),
                    "authentication".to_string(),
                    "input validation".to_string(),
                    "cryptography".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Performance Reviewer".to_string(),
                instructions: "Identify performance bottlenecks, inefficient algorithms, memory leaks, and resource management issues. Suggest optimizations and note any O(n²) or worse complexity.".to_string(),
                expertise: vec![
                    "performance".to_string(),
                    "algorithms".to_string(),
                    "memory management".to_string(),
                    "profiling".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Maintainability Reviewer".to_string(),
                instructions: "Evaluate code readability, documentation quality, test coverage, and adherence to coding standards. Suggest improvements for long-term maintainability.".to_string(),
                expertise: vec![
                    "clean code".to_string(),
                    "testing".to_string(),
                    "documentation".to_string(),
                    "design patterns".to_string(),
                ],
            },
        ],
        context: Some(TemplateContext {
            prefix: Some("Review the following code for issues and improvements:".to_string()),
            suffix: Some("Provide specific line references where applicable.".to_string()),
        }),
    }
}

fn architecture_decision_template() -> DebateTemplate {
    DebateTemplate {
        name: "architecture-decision".to_string(),
        description: "Evaluate architectural choices from multiple stakeholder perspectives".to_string(),
        default_rounds: 3,
        participants: vec![
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "System Architect".to_string(),
                instructions: "Evaluate technical feasibility, scalability, and long-term maintainability. Consider system boundaries, data flow, and integration points.".to_string(),
                expertise: vec![
                    "system design".to_string(),
                    "scalability".to_string(),
                    "distributed systems".to_string(),
                    "microservices".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Senior Developer".to_string(),
                instructions: "Focus on implementation complexity, developer experience, and day-to-day maintainability. Consider the impact on development velocity and debugging.".to_string(),
                expertise: vec![
                    "implementation".to_string(),
                    "debugging".to_string(),
                    "developer experience".to_string(),
                    "code organization".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Operations Engineer".to_string(),
                instructions: "Evaluate operational concerns: deployment, monitoring, incident response, and infrastructure costs. Consider failure modes and recovery procedures.".to_string(),
                expertise: vec![
                    "DevOps".to_string(),
                    "monitoring".to_string(),
                    "infrastructure".to_string(),
                    "incident response".to_string(),
                ],
            },
        ],
        context: None,
    }
}

fn security_audit_template() -> DebateTemplate {
    DebateTemplate {
        name: "security-audit".to_string(),
        description: "Security-focused analysis from CISO, security engineer, and compliance perspectives".to_string(),
        default_rounds: 2,
        participants: vec![
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "CISO".to_string(),
                instructions: "Evaluate business risk, regulatory compliance, and strategic security posture. Consider reputational impact and risk tolerance.".to_string(),
                expertise: vec![
                    "risk management".to_string(),
                    "compliance".to_string(),
                    "security strategy".to_string(),
                    "incident response".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Security Engineer".to_string(),
                instructions: "Perform technical security analysis: identify vulnerabilities, attack vectors, and remediation steps. Reference CVEs and security best practices.".to_string(),
                expertise: vec![
                    "penetration testing".to_string(),
                    "vulnerability assessment".to_string(),
                    "secure coding".to_string(),
                    "threat modeling".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Compliance Officer".to_string(),
                instructions: "Evaluate compliance with relevant regulations (GDPR, SOC2, HIPAA, PCI-DSS). Identify gaps and required documentation.".to_string(),
                expertise: vec![
                    "GDPR".to_string(),
                    "SOC2".to_string(),
                    "audit requirements".to_string(),
                    "data protection".to_string(),
                ],
            },
        ],
        context: Some(TemplateContext {
            prefix: Some("Conduct a security audit of the following:".to_string()),
            suffix: Some("Prioritize findings by severity (Critical, High, Medium, Low).".to_string()),
        }),
    }
}

fn api_design_template() -> DebateTemplate {
    DebateTemplate {
        name: "api-design".to_string(),
        description: "API design review from backend, frontend, and API consumer perspectives".to_string(),
        default_rounds: 2,
        participants: vec![
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Backend Engineer".to_string(),
                instructions: "Evaluate API design from implementation perspective: data modeling, query efficiency, caching strategies, and backend scalability.".to_string(),
                expertise: vec![
                    "REST".to_string(),
                    "GraphQL".to_string(),
                    "database design".to_string(),
                    "caching".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Frontend Engineer".to_string(),
                instructions: "Evaluate API design from consumer perspective: ease of use, response structure, error handling, and data loading patterns.".to_string(),
                expertise: vec![
                    "API consumption".to_string(),
                    "state management".to_string(),
                    "error handling".to_string(),
                    "user experience".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "API Platform Lead".to_string(),
                instructions: "Evaluate API design for consistency, versioning strategy, documentation quality, and adherence to API design standards.".to_string(),
                expertise: vec![
                    "API governance".to_string(),
                    "versioning".to_string(),
                    "documentation".to_string(),
                    "developer experience".to_string(),
                ],
            },
        ],
        context: Some(TemplateContext {
            prefix: Some("Review the following API design:".to_string()),
            suffix: None,
        }),
    }
}

fn incident_postmortem_template() -> DebateTemplate {
    DebateTemplate {
        name: "incident-postmortem".to_string(),
        description: "Incident analysis from SRE, developer, and product perspectives".to_string(),
        default_rounds: 2,
        participants: vec![
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "SRE".to_string(),
                instructions: "Analyze the incident from operational perspective: detection time, response procedures, monitoring gaps, and infrastructure improvements.".to_string(),
                expertise: vec![
                    "incident response".to_string(),
                    "monitoring".to_string(),
                    "SLOs".to_string(),
                    "runbooks".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Developer".to_string(),
                instructions: "Analyze the incident from code perspective: root cause, code changes needed, testing gaps, and prevention strategies.".to_string(),
                expertise: vec![
                    "debugging".to_string(),
                    "root cause analysis".to_string(),
                    "testing".to_string(),
                    "code review".to_string(),
                ],
            },
            TemplateParticipant {
                cli: "claude".to_string(),
                persona: "Product Manager".to_string(),
                instructions: "Analyze the incident from user impact perspective: affected features, communication strategy, and prioritization of fixes.".to_string(),
                expertise: vec![
                    "user impact".to_string(),
                    "prioritization".to_string(),
                    "stakeholder communication".to_string(),
                    "roadmap".to_string(),
                ],
            },
        ],
        context: Some(TemplateContext {
            prefix: Some("Analyze the following incident:".to_string()),
            suffix: Some("Propose action items with owners and timelines.".to_string()),
        }),
    }
}
