//! 工具 Trait 定义
//! 
//! 这个模块定义了工具的核心 trait

use crate::error::Result;
use async_trait::async_trait;
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ValidationResult,
    PermissionResult, ToolInputSchema,
};

/// 工具 Trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// 获取工具元数据
    fn metadata(&self) -> ToolMetadata;
    
    /// 获取工具名称
    fn name(&self) -> String {
        self.metadata().name
    }
    
    /// 获取工具描述
    fn description(&self) -> String {
        self.metadata().description
    }
    
    /// 验证输入
    async fn validate_input(
        &self,
        _input: &serde_json::Value,
        _context: &ToolUseContext,
    ) -> Result<ValidationResult> {
        Ok(ValidationResult::valid())
    }
    
    /// 检查权限
    async fn check_permissions(
        &self,
        input: &serde_json::Value,
        context: &ToolUseContext,
    ) -> Result<PermissionResult> {
        // 默认实现：使用权限检查器
        let result = super::permissions::PermissionChecker::check(
            &self.name(),
            input,
            &context.permission_context,
        );
        Ok(result)
    }
    
    /// 执行工具
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult>;
    
    /// 是否启用
    fn is_enabled(&self) -> bool {
        self.metadata().is_enabled
    }
    
    /// 是否只读
    fn is_read_only(&self) -> bool {
        self.metadata().is_read_only
    }
    
    /// 是否破坏性
    fn is_destructive(&self) -> bool {
        self.metadata().is_destructive
    }
    
    /// 是否并发安全
    fn is_concurrency_safe(&self, _input: &serde_json::Value) -> bool {
        false
    }
    
    /// 获取输入 Schema
    fn input_schema(&self) -> ToolInputSchema {
        self.metadata().input_schema
    }
    
    /// 获取工具路径（如果工具操作文件路径）
    fn get_path(&self, _input: &serde_json::Value) -> Option<String> {
        None
    }
    
    /// 用户友好的名称
    fn user_facing_name(&self, _input: &serde_json::Value) -> String {
        self.name()
    }
    
    /// 获取活动描述
    fn get_activity_description(&self, _input: &serde_json::Value) -> Option<String> {
        None
    }
    
    /// 匹配工具名称（包括别名）
    fn matches_name(&self, name: &str) -> bool {
        let metadata = self.metadata();
        if metadata.name == name {
            return true;
        }
        
        if let Some(aliases) = &metadata.aliases {
            return aliases.contains(&name.to_string());
        }
        
        false
    }
}

/// 工具构建器
pub struct ToolBuilder {
    name: String,
    description: String,
    category: super::types::ToolCategory,
    permission_level: super::types::ToolPermissionLevel,
    aliases: Option<Vec<String>>,
    is_read_only: bool,
    is_destructive: bool,
    is_enabled: bool,
    input_schema: ToolInputSchema,
}

impl ToolBuilder {
    /// 创建新的工具构建器
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: super::types::ToolCategory::Other,
            permission_level: super::types::ToolPermissionLevel::Standard,
            aliases: None,
            is_read_only: false,
            is_destructive: false,
            is_enabled: true,
            input_schema: ToolInputSchema::default(),
        }
    }
    
    /// 设置类别
    pub fn category(mut self, category: super::types::ToolCategory) -> Self {
        self.category = category;
        self
    }
    
    /// 设置权限级别
    pub fn permission_level(mut self, level: super::types::ToolPermissionLevel) -> Self {
        self.permission_level = level;
        self
    }
    
    /// 设置别名
    pub fn aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = Some(aliases);
        self
    }
    
    /// 设置为只读
    pub fn read_only(mut self) -> Self {
        self.is_read_only = true;
        self
    }
    
    /// 设置为破坏性
    pub fn destructive(mut self) -> Self {
        self.is_destructive = true;
        self
    }
    
    /// 设置是否启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }
    
    /// 设置输入 Schema
    pub fn input_schema(mut self, schema: ToolInputSchema) -> Self {
        self.input_schema = schema;
        self
    }
    
    /// 构建元数据
    pub fn build_metadata(self) -> ToolMetadata {
        ToolMetadata {
            name: self.name,
            description: self.description,
            category: self.category,
            permission_level: self.permission_level,
            aliases: self.aliases,
            is_read_only: self.is_read_only,
            is_destructive: self.is_destructive,
            is_enabled: self.is_enabled,
            is_mcp: None,
            input_schema: self.input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestTool;
    
    #[async_trait]
    impl Tool for TestTool {
        fn metadata(&self) -> ToolMetadata {
            ToolBuilder::new("test", "Test tool")
                .category(ToolCategory::Other)
                .build_metadata()
        }
        
        async fn execute(
            &self,
            _input: serde_json::Value,
            _context: ToolUseContext,
        ) -> Result<ToolResult> {
            Ok(ToolResult::success(serde_json::json!({"result": "ok"})))
        }
    }
    
    #[test]
    fn test_tool_metadata() {
        let tool = TestTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, "Test tool");
        assert_eq!(metadata.category, types::ToolCategory::Other);
    }
    
    #[test]
    fn test_tool_matches_name() {
        let tool = TestTool;
        
        assert!(tool.matches_name("test"));
        assert!(!tool.matches_name("other"));
    }
    
    #[test]
    fn test_tool_builder() {
        let metadata = ToolBuilder::new("read", "Read file")
            .category(types::ToolCategory::FileOperation)
            .permission_level(types::ToolPermissionLevel::Standard)
            .aliases(vec!["r".to_string()])
            .read_only()
            .build_metadata();
        
        assert_eq!(metadata.name, "read");
        assert_eq!(metadata.category, types::ToolCategory::FileOperation);
        assert!(metadata.is_read_only);
        assert_eq!(metadata.aliases, Some(vec!["r".to_string()]));
    }
}
