use super::*;
use serde_yaml;

#[test]
fn test_open_design_metadata_default() {
    let metadata = OpenDesignMetadata::default();
    assert!(metadata.mode.is_none());
    assert!(metadata.platform.is_none());
    assert!(metadata.scenario.is_none());
    assert!(metadata.preview.is_none());
    assert!(metadata.design_system.is_none());
    assert!(metadata.inputs.is_none());
    assert!(metadata.outputs.is_none());
    assert!(metadata.capabilities_required.is_none());
    assert!(metadata.category.is_none());
    assert!(metadata.upstream.is_none());
    assert!(metadata.triggers.is_none());
}

#[test]
fn test_open_design_metadata_deserialize() {
    let yaml_str = r#"
mode: preview
platform: web
scenario: design-review
category: design
"#;
    
    let metadata: OpenDesignMetadata = serde_yaml::from_str(yaml_str).unwrap();
    
    assert_eq!(metadata.mode, Some("preview".to_string()));
    assert_eq!(metadata.platform, Some("web".to_string()));
    assert_eq!(metadata.scenario, Some("design-review".to_string()));
    assert_eq!(metadata.category, Some("design".to_string()));
}

#[test]
fn test_preview_config_default() {
    let config = PreviewConfig::default();
    assert!(config.reload_strategy.is_none());
    assert_eq!(config.debounce_ms, 500);
}

#[test]
fn test_preview_config_deserialize() {
    let yaml_str = r#"
reload_strategy: debounce
debounce_ms: 1000
"#;
    
    let config: PreviewConfig = serde_yaml::from_str(yaml_str).unwrap();
    
    assert_eq!(config.reload_strategy, Some("debounce".to_string()));
    assert_eq!(config.debounce_ms, 1000);
}

#[test]
fn test_input_field_default() {
    let field = InputField::default();
    assert_eq!(field.name, "");
    assert_eq!(field.r#type, "");
    assert!(field.label.is_none());
    assert!(field.description.is_none());
    assert!(field.required.is_none());
    assert!(field.default.is_none());
}

#[test]
fn test_input_field_deserialize() {
    let yaml_str = r#"
name: prompt
type: string
label: Design Prompt
description: Description of the design
required: true
default: Create a website
"#;
    
    let field: InputField = serde_yaml::from_str(yaml_str).unwrap();
    
    assert_eq!(field.name, "prompt");
    assert_eq!(field.r#type, "string");
    assert_eq!(field.label, Some("Design Prompt".to_string()));
    assert_eq!(field.description, Some("Description of the design".to_string()));
    assert_eq!(field.required, Some(true));
    assert_eq!(field.default, Some("Create a website".to_string()));
}

#[test]
fn test_design_system_config_default() {
    let config = DesignSystemConfig::default();
    assert!(config.tokens.is_none());
    assert!(config.theme.is_none());
}

#[test]
fn test_output_config_default() {
    let config = OutputConfig::default();
    assert!(config.format.is_none());
    assert!(config.quality.is_none());
}

#[test]
fn test_full_open_design_metadata() {
    let yaml_str = r#"
mode: generation
platform: mobile
scenario: ui-prototype
category: design
preview:
  reload_strategy: immediate
  debounce_ms: 300
inputs:
  - name: prompt
    type: string
    label: Design Prompt
    required: true
outputs:
  format: html
  quality: high
capabilities_required:
  - html-generation
  - css-styling
triggers:
  - design-request
"#;
    
    let metadata: OpenDesignMetadata = serde_yaml::from_str(yaml_str).unwrap();
    
    assert_eq!(metadata.mode, Some("generation".to_string()));
    assert_eq!(metadata.platform, Some("mobile".to_string()));
    assert_eq!(metadata.category, Some("design".to_string()));
    assert!(metadata.preview.is_some());
    assert_eq!(metadata.preview.as_ref().unwrap().debounce_ms, 300);
    assert!(metadata.inputs.is_some());
    assert_eq!(metadata.inputs.as_ref().unwrap().len(), 1);
    assert!(metadata.capabilities_required.is_some());
    assert_eq!(metadata.capabilities_required.as_ref().unwrap().len(), 2);
}
