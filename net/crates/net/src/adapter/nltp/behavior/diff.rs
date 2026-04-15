//! Capability Change Diffs (CAP-DIFF) for Phase 4B.
//!
//! This module provides:
//! - `DiffOp` - Individual diff operations (add/remove tags, models, tools, etc.)
//! - `CapabilityDiff` - Versioned diff message with operations
//! - `DiffEngine` - Generate and apply diffs between capability sets
//!
//! # Performance Targets
//! - Diff generation: < 1µs for typical changes
//! - Diff application: < 500ns
//! - Diff size (1 op): < 50 bytes
//! - Bandwidth savings: > 90% vs full CAP-ANN

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::capability::{
    CapabilitySet, HardwareCapabilities, ModelCapability, ResourceLimits, SoftwareCapabilities,
    ToolCapability,
};

// ============================================================================
// Diff Operations
// ============================================================================

/// Individual diff operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffOp {
    // Tag operations
    /// Add a tag
    AddTag(String),
    /// Remove a tag
    RemoveTag(String),

    // Model operations
    /// Add a model capability
    AddModel(ModelCapability),
    /// Remove a model by ID
    RemoveModel(String),
    /// Update model fields (partial update)
    UpdateModel {
        /// Model ID to update
        model_id: String,
        /// New tokens per second (if changed)
        tokens_per_sec: Option<u32>,
        /// New loaded status (if changed)
        loaded: Option<bool>,
    },

    // Tool operations
    /// Add a tool capability
    AddTool(ToolCapability),
    /// Remove a tool by ID
    RemoveTool(String),

    // Hardware operations
    /// Update hardware capabilities (full replacement)
    UpdateHardware(HardwareCapabilities),
    /// Update memory only
    UpdateMemory(u32),
    /// Update network bandwidth only
    UpdateNetwork(u32),

    // Software operations
    /// Update software capabilities (full replacement)
    UpdateSoftware(SoftwareCapabilities),
    /// Add a runtime
    AddRuntime {
        /// Runtime name
        name: String,
        /// Runtime version
        version: String,
    },
    /// Remove a runtime
    RemoveRuntime(String),
    /// Add a framework
    AddFramework {
        /// Framework name
        name: String,
        /// Framework version
        version: String,
    },
    /// Remove a framework
    RemoveFramework(String),

    // Resource limits
    /// Update resource limits (full replacement)
    UpdateLimits(ResourceLimits),
    /// Update max concurrent requests only
    UpdateMaxConcurrent(u32),
    /// Update rate limit only
    UpdateRateLimit(u32),

    // Custom field operations (for extensibility)
    /// Set a custom JSON field by path
    SetField {
        /// JSON path (e.g., "custom.foo.bar")
        path: String,
        /// JSON value
        value: serde_json::Value,
    },
    /// Unset a custom field
    UnsetField {
        /// JSON path
        path: String,
    },
}

impl DiffOp {
    /// Estimate serialized size of this operation in bytes
    pub fn estimated_size(&self) -> usize {
        match self {
            DiffOp::AddTag(s) | DiffOp::RemoveTag(s) => 8 + s.len(),
            DiffOp::AddModel(m) => 50 + m.model_id.len() + m.family.len(),
            DiffOp::RemoveModel(s) => 8 + s.len(),
            DiffOp::UpdateModel { model_id, .. } => 16 + model_id.len(),
            DiffOp::AddTool(t) => 50 + t.tool_id.len() + t.name.len(),
            DiffOp::RemoveTool(s) => 8 + s.len(),
            DiffOp::UpdateHardware(_) => 64,
            DiffOp::UpdateMemory(_) => 8,
            DiffOp::UpdateNetwork(_) => 8,
            DiffOp::UpdateSoftware(_) => 128,
            DiffOp::AddRuntime { name, version } => 12 + name.len() + version.len(),
            DiffOp::RemoveRuntime(s) => 8 + s.len(),
            DiffOp::AddFramework { name, version } => 12 + name.len() + version.len(),
            DiffOp::RemoveFramework(s) => 8 + s.len(),
            DiffOp::UpdateLimits(_) => 32,
            DiffOp::UpdateMaxConcurrent(_) => 8,
            DiffOp::UpdateRateLimit(_) => 8,
            DiffOp::SetField { path, value } => 16 + path.len() + value.to_string().len(),
            DiffOp::UnsetField { path } => 8 + path.len(),
        }
    }

    /// Check if this is a tag operation
    pub fn is_tag_op(&self) -> bool {
        matches!(self, DiffOp::AddTag(_) | DiffOp::RemoveTag(_))
    }

    /// Check if this is a model operation
    pub fn is_model_op(&self) -> bool {
        matches!(
            self,
            DiffOp::AddModel(_) | DiffOp::RemoveModel(_) | DiffOp::UpdateModel { .. }
        )
    }

    /// Check if this is a tool operation
    pub fn is_tool_op(&self) -> bool {
        matches!(self, DiffOp::AddTool(_) | DiffOp::RemoveTool(_))
    }
}

// ============================================================================
// Capability Diff
// ============================================================================

/// Capability diff message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiff {
    /// Source node ID
    pub node_id: u64,
    /// Base version this diff applies to
    pub base_version: u64,
    /// New version after applying diff
    pub new_version: u64,
    /// Operations to apply (in order)
    pub ops: Vec<DiffOp>,
    /// Timestamp (nanoseconds since epoch)
    pub timestamp_ns: u64,
}

impl CapabilityDiff {
    /// Create a new capability diff
    pub fn new(node_id: u64, base_version: u64, new_version: u64, ops: Vec<DiffOp>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        Self {
            node_id,
            base_version,
            new_version,
            ops,
            timestamp_ns,
        }
    }

    /// Check if this diff is empty (no operations)
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Get number of operations
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Estimate total serialized size in bytes
    pub fn estimated_size(&self) -> usize {
        // Header: node_id(8) + base_version(8) + new_version(8) + timestamp(8) + ops_len(4)
        let header_size = 36;
        let ops_size: usize = self.ops.iter().map(|op| op.estimated_size()).sum();
        header_size + ops_size
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

// ============================================================================
// Diff Error
// ============================================================================

/// Error during diff application
#[derive(Debug, Clone, PartialEq)]
pub enum DiffError {
    /// Version mismatch (expected base version doesn't match)
    VersionMismatch {
        /// Expected base version
        expected: u64,
        /// Actual current version
        actual: u64,
    },
    /// Model not found for update/remove
    ModelNotFound(String),
    /// Tool not found for remove
    ToolNotFound(String),
    /// Tag not found for remove
    TagNotFound(String),
    /// Runtime not found for remove
    RuntimeNotFound(String),
    /// Framework not found for remove
    FrameworkNotFound(String),
    /// Invalid field path
    InvalidFieldPath(String),
    /// Operation not applicable
    NotApplicable(String),
}

impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffError::VersionMismatch { expected, actual } => {
                write!(f, "version mismatch: expected {}, got {}", expected, actual)
            }
            DiffError::ModelNotFound(id) => write!(f, "model not found: {}", id),
            DiffError::ToolNotFound(id) => write!(f, "tool not found: {}", id),
            DiffError::TagNotFound(tag) => write!(f, "tag not found: {}", tag),
            DiffError::RuntimeNotFound(name) => write!(f, "runtime not found: {}", name),
            DiffError::FrameworkNotFound(name) => write!(f, "framework not found: {}", name),
            DiffError::InvalidFieldPath(path) => write!(f, "invalid field path: {}", path),
            DiffError::NotApplicable(msg) => write!(f, "operation not applicable: {}", msg),
        }
    }
}

impl std::error::Error for DiffError {}

// ============================================================================
// Diff Engine
// ============================================================================

/// Engine for generating and applying capability diffs
pub struct DiffEngine;

impl DiffEngine {
    /// Generate diff operations between old and new capability sets
    pub fn diff(old: &CapabilitySet, new: &CapabilitySet) -> Vec<DiffOp> {
        let mut ops = Vec::new();

        // Diff tags
        Self::diff_tags(&old.tags, &new.tags, &mut ops);

        // Diff models
        Self::diff_models(&old.models, &new.models, &mut ops);

        // Diff tools
        Self::diff_tools(&old.tools, &new.tools, &mut ops);

        // Diff hardware (only if changed)
        if old.hardware != new.hardware {
            // Check for partial updates
            if old.hardware.memory_mb != new.hardware.memory_mb
                && old.hardware.cpu_cores == new.hardware.cpu_cores
                && old.hardware.gpu == new.hardware.gpu
                && old.hardware.storage_mb == new.hardware.storage_mb
                && old.hardware.network_mbps == new.hardware.network_mbps
            {
                ops.push(DiffOp::UpdateMemory(new.hardware.memory_mb));
            } else if old.hardware.network_mbps != new.hardware.network_mbps
                && old.hardware.cpu_cores == new.hardware.cpu_cores
                && old.hardware.gpu == new.hardware.gpu
                && old.hardware.memory_mb == new.hardware.memory_mb
                && old.hardware.storage_mb == new.hardware.storage_mb
            {
                ops.push(DiffOp::UpdateNetwork(new.hardware.network_mbps));
            } else {
                ops.push(DiffOp::UpdateHardware(new.hardware.clone()));
            }
        }

        // Diff software
        if old.software != new.software {
            Self::diff_software(&old.software, &new.software, &mut ops);
        }

        // Diff limits (only if changed)
        if old.limits != new.limits {
            // Check for partial updates
            if old.limits.max_concurrent_requests != new.limits.max_concurrent_requests
                && old.limits.max_tokens_per_request == new.limits.max_tokens_per_request
                && old.limits.rate_limit_rpm == new.limits.rate_limit_rpm
                && old.limits.max_batch_size == new.limits.max_batch_size
            {
                ops.push(DiffOp::UpdateMaxConcurrent(
                    new.limits.max_concurrent_requests,
                ));
            } else if old.limits.rate_limit_rpm != new.limits.rate_limit_rpm
                && old.limits.max_concurrent_requests == new.limits.max_concurrent_requests
                && old.limits.max_tokens_per_request == new.limits.max_tokens_per_request
                && old.limits.max_batch_size == new.limits.max_batch_size
            {
                ops.push(DiffOp::UpdateRateLimit(new.limits.rate_limit_rpm));
            } else {
                ops.push(DiffOp::UpdateLimits(new.limits.clone()));
            }
        }

        ops
    }

    /// Diff tags between old and new
    fn diff_tags(old: &[String], new: &[String], ops: &mut Vec<DiffOp>) {
        let old_set: HashSet<&str> = old.iter().map(|s| s.as_str()).collect();
        let new_set: HashSet<&str> = new.iter().map(|s| s.as_str()).collect();

        // Removed tags
        for tag in old_set.difference(&new_set) {
            ops.push(DiffOp::RemoveTag((*tag).to_string()));
        }

        // Added tags
        for tag in new_set.difference(&old_set) {
            ops.push(DiffOp::AddTag((*tag).to_string()));
        }
    }

    /// Diff models between old and new
    fn diff_models(old: &[ModelCapability], new: &[ModelCapability], ops: &mut Vec<DiffOp>) {
        let old_map: std::collections::HashMap<&str, &ModelCapability> =
            old.iter().map(|m| (m.model_id.as_str(), m)).collect();
        let new_map: std::collections::HashMap<&str, &ModelCapability> =
            new.iter().map(|m| (m.model_id.as_str(), m)).collect();

        // Removed models
        for (id, _) in old_map.iter() {
            if !new_map.contains_key(id) {
                ops.push(DiffOp::RemoveModel((*id).to_string()));
            }
        }

        // Added or updated models
        for (id, new_model) in new_map.iter() {
            if let Some(old_model) = old_map.get(id) {
                // Check for updates
                if *old_model != *new_model {
                    // Check if only tokens_per_sec or loaded changed
                    if old_model.family == new_model.family
                        && old_model.parameters_b_x10 == new_model.parameters_b_x10
                        && old_model.context_length == new_model.context_length
                        && old_model.quantization == new_model.quantization
                        && old_model.modalities == new_model.modalities
                    {
                        // Partial update
                        let tokens_per_sec = if old_model.tokens_per_sec != new_model.tokens_per_sec
                        {
                            Some(new_model.tokens_per_sec)
                        } else {
                            None
                        };
                        let loaded = if old_model.loaded != new_model.loaded {
                            Some(new_model.loaded)
                        } else {
                            None
                        };
                        if tokens_per_sec.is_some() || loaded.is_some() {
                            ops.push(DiffOp::UpdateModel {
                                model_id: (*id).to_string(),
                                tokens_per_sec,
                                loaded,
                            });
                        }
                    } else {
                        // Full replacement
                        ops.push(DiffOp::RemoveModel((*id).to_string()));
                        ops.push(DiffOp::AddModel((*new_model).clone()));
                    }
                }
            } else {
                // New model
                ops.push(DiffOp::AddModel((*new_model).clone()));
            }
        }
    }

    /// Diff tools between old and new
    fn diff_tools(old: &[ToolCapability], new: &[ToolCapability], ops: &mut Vec<DiffOp>) {
        let old_map: std::collections::HashMap<&str, &ToolCapability> =
            old.iter().map(|t| (t.tool_id.as_str(), t)).collect();
        let new_map: std::collections::HashMap<&str, &ToolCapability> =
            new.iter().map(|t| (t.tool_id.as_str(), t)).collect();

        // Removed tools
        for (id, _) in old_map.iter() {
            if !new_map.contains_key(id) {
                ops.push(DiffOp::RemoveTool((*id).to_string()));
            }
        }

        // Added or replaced tools
        for (id, new_tool) in new_map.iter() {
            if let Some(old_tool) = old_map.get(id) {
                if *old_tool != *new_tool {
                    // Tools don't have partial updates, full replacement
                    ops.push(DiffOp::RemoveTool((*id).to_string()));
                    ops.push(DiffOp::AddTool((*new_tool).clone()));
                }
            } else {
                ops.push(DiffOp::AddTool((*new_tool).clone()));
            }
        }
    }

    /// Diff software capabilities
    fn diff_software(
        old: &SoftwareCapabilities,
        new: &SoftwareCapabilities,
        ops: &mut Vec<DiffOp>,
    ) {
        // Check if we can do partial updates
        let os_changed = old.os != new.os || old.os_version != new.os_version;
        let cuda_changed = old.cuda_version != new.cuda_version;
        let drivers_changed = old.drivers != new.drivers;

        if os_changed || cuda_changed || drivers_changed {
            // Full software update needed
            ops.push(DiffOp::UpdateSoftware(new.clone()));
            return;
        }

        // Diff runtimes
        let old_runtimes: std::collections::HashMap<&str, &str> = old
            .runtimes
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        let new_runtimes: std::collections::HashMap<&str, &str> = new
            .runtimes
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();

        for (name, _) in old_runtimes.iter() {
            if !new_runtimes.contains_key(name) {
                ops.push(DiffOp::RemoveRuntime((*name).to_string()));
            }
        }
        for (name, version) in new_runtimes.iter() {
            if old_runtimes.get(name) != Some(version) {
                ops.push(DiffOp::AddRuntime {
                    name: (*name).to_string(),
                    version: (*version).to_string(),
                });
            }
        }

        // Diff frameworks
        let old_frameworks: std::collections::HashMap<&str, &str> = old
            .frameworks
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        let new_frameworks: std::collections::HashMap<&str, &str> = new
            .frameworks
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();

        for (name, _) in old_frameworks.iter() {
            if !new_frameworks.contains_key(name) {
                ops.push(DiffOp::RemoveFramework((*name).to_string()));
            }
        }
        for (name, version) in new_frameworks.iter() {
            if old_frameworks.get(name) != Some(version) {
                ops.push(DiffOp::AddFramework {
                    name: (*name).to_string(),
                    version: (*version).to_string(),
                });
            }
        }
    }

    /// Apply diff operations to a capability set
    ///
    /// Returns the updated capability set or an error if application fails.
    /// The `strict` parameter controls whether missing items cause errors.
    pub fn apply(
        base: &CapabilitySet,
        diff: &CapabilityDiff,
        strict: bool,
    ) -> Result<CapabilitySet, DiffError> {
        let mut result = base.clone();

        for op in &diff.ops {
            Self::apply_op(&mut result, op, strict)?;
        }

        Ok(result)
    }

    /// Apply a single diff operation
    fn apply_op(caps: &mut CapabilitySet, op: &DiffOp, strict: bool) -> Result<(), DiffError> {
        match op {
            DiffOp::AddTag(tag) => {
                if !caps.tags.contains(tag) {
                    caps.tags.push(tag.clone());
                }
            }
            DiffOp::RemoveTag(tag) => {
                if let Some(pos) = caps.tags.iter().position(|t| t == tag) {
                    caps.tags.remove(pos);
                } else if strict {
                    return Err(DiffError::TagNotFound(tag.clone()));
                }
            }
            DiffOp::AddModel(model) => {
                // Remove existing model with same ID if present
                caps.models.retain(|m| m.model_id != model.model_id);
                caps.models.push(model.clone());
            }
            DiffOp::RemoveModel(model_id) => {
                let before = caps.models.len();
                caps.models.retain(|m| m.model_id != *model_id);
                if strict && caps.models.len() == before {
                    return Err(DiffError::ModelNotFound(model_id.clone()));
                }
            }
            DiffOp::UpdateModel {
                model_id,
                tokens_per_sec,
                loaded,
            } => {
                if let Some(model) = caps.models.iter_mut().find(|m| m.model_id == *model_id) {
                    if let Some(tps) = tokens_per_sec {
                        model.tokens_per_sec = *tps;
                    }
                    if let Some(l) = loaded {
                        model.loaded = *l;
                    }
                } else if strict {
                    return Err(DiffError::ModelNotFound(model_id.clone()));
                }
            }
            DiffOp::AddTool(tool) => {
                // Remove existing tool with same ID if present
                caps.tools.retain(|t| t.tool_id != tool.tool_id);
                caps.tools.push(tool.clone());
            }
            DiffOp::RemoveTool(tool_id) => {
                let before = caps.tools.len();
                caps.tools.retain(|t| t.tool_id != *tool_id);
                if strict && caps.tools.len() == before {
                    return Err(DiffError::ToolNotFound(tool_id.clone()));
                }
            }
            DiffOp::UpdateHardware(hw) => {
                caps.hardware = hw.clone();
            }
            DiffOp::UpdateMemory(mem) => {
                caps.hardware.memory_mb = *mem;
            }
            DiffOp::UpdateNetwork(net) => {
                caps.hardware.network_mbps = *net;
            }
            DiffOp::UpdateSoftware(sw) => {
                caps.software = sw.clone();
            }
            DiffOp::AddRuntime { name, version } => {
                // Remove existing runtime with same name
                caps.software.runtimes.retain(|(n, _)| n != name);
                caps.software.runtimes.push((name.clone(), version.clone()));
            }
            DiffOp::RemoveRuntime(name) => {
                let before = caps.software.runtimes.len();
                caps.software.runtimes.retain(|(n, _)| n != name);
                if strict && caps.software.runtimes.len() == before {
                    return Err(DiffError::RuntimeNotFound(name.clone()));
                }
            }
            DiffOp::AddFramework { name, version } => {
                // Remove existing framework with same name
                caps.software.frameworks.retain(|(n, _)| n != name);
                caps.software
                    .frameworks
                    .push((name.clone(), version.clone()));
            }
            DiffOp::RemoveFramework(name) => {
                let before = caps.software.frameworks.len();
                caps.software.frameworks.retain(|(n, _)| n != name);
                if strict && caps.software.frameworks.len() == before {
                    return Err(DiffError::FrameworkNotFound(name.clone()));
                }
            }
            DiffOp::UpdateLimits(limits) => {
                caps.limits = limits.clone();
            }
            DiffOp::UpdateMaxConcurrent(max) => {
                caps.limits.max_concurrent_requests = *max;
            }
            DiffOp::UpdateRateLimit(rpm) => {
                caps.limits.rate_limit_rpm = *rpm;
            }
            DiffOp::SetField { path: _, value: _ } => {
                // Custom field operations would require JSON manipulation
                // For now, this is a no-op placeholder
            }
            DiffOp::UnsetField { path: _ } => {
                // Custom field operations would require JSON manipulation
                // For now, this is a no-op placeholder
            }
        }
        Ok(())
    }

    /// Validate that a chain of diffs is consistent
    ///
    /// Checks that version numbers are sequential and base versions match.
    pub fn validate_chain(diffs: &[CapabilityDiff]) -> bool {
        if diffs.is_empty() {
            return true;
        }

        for i in 1..diffs.len() {
            let prev = &diffs[i - 1];
            let curr = &diffs[i];

            // Same node
            if prev.node_id != curr.node_id {
                return false;
            }

            // Version chain
            if prev.new_version != curr.base_version {
                return false;
            }

            // Monotonic timestamps
            if prev.timestamp_ns > curr.timestamp_ns {
                return false;
            }
        }

        true
    }

    /// Compact a chain of diffs into a single diff
    ///
    /// This is useful for reducing storage/bandwidth when many small diffs accumulate.
    pub fn compact(base: &CapabilitySet, diffs: &[CapabilityDiff]) -> Option<CapabilityDiff> {
        if diffs.is_empty() {
            return None;
        }

        // Apply all diffs to get final state
        let mut current = base.clone();
        for diff in diffs {
            current = Self::apply(&current, diff, false).ok()?;
        }

        // Generate new diff from base to final
        let ops = Self::diff(base, &current);

        let first = diffs.first()?;
        let last = diffs.last()?;

        Some(CapabilityDiff {
            node_id: first.node_id,
            base_version: first.base_version,
            new_version: last.new_version,
            ops,
            timestamp_ns: last.timestamp_ns,
        })
    }

    /// Estimate bandwidth savings of using diff vs full announcement
    ///
    /// Returns (diff_size, full_size, savings_percent)
    pub fn bandwidth_savings(diff: &CapabilityDiff, full: &CapabilitySet) -> (usize, usize, f64) {
        let diff_size = diff.estimated_size();
        let full_size = full.to_bytes().len();

        let savings = if full_size > 0 {
            100.0 * (1.0 - (diff_size as f64 / full_size as f64))
        } else {
            0.0
        };

        (diff_size, full_size, savings)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::nltp::behavior::capability::{GpuInfo, GpuVendor, Modality};

    fn sample_capability_set() -> CapabilitySet {
        let gpu = GpuInfo::new(GpuVendor::Nvidia, "RTX 4090", 24576);
        let hardware = HardwareCapabilities::new()
            .with_cpu(16, 32)
            .with_memory(65536)
            .with_gpu(gpu);

        let software = SoftwareCapabilities::new()
            .with_os("linux", "6.1")
            .add_runtime("python", "3.11")
            .add_framework("pytorch", "2.1");

        let model = ModelCapability::new("llama-3.1-70b", "llama")
            .with_parameters(70.0)
            .with_context_length(128000)
            .add_modality(Modality::Text)
            .with_tokens_per_sec(50)
            .with_loaded(true);

        let tool = ToolCapability::new("python_repl", "Python REPL");

        CapabilitySet::new()
            .with_hardware(hardware)
            .with_software(software)
            .add_model(model)
            .add_tool(tool)
            .add_tag("inference")
            .add_tag("gpu")
            .with_limits(ResourceLimits::new().with_max_concurrent(10))
    }

    #[test]
    fn test_diff_no_changes() {
        let caps = sample_capability_set();
        let ops = DiffEngine::diff(&caps, &caps);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_diff_add_tag() {
        let old = sample_capability_set();
        let mut new = old.clone();
        new.tags.push("training".into());

        let ops = DiffEngine::diff(&old, &new);
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], DiffOp::AddTag(t) if t == "training"));
    }

    #[test]
    fn test_diff_remove_tag() {
        let old = sample_capability_set();
        let mut new = old.clone();
        new.tags.retain(|t| t != "inference");

        let ops = DiffEngine::diff(&old, &new);
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], DiffOp::RemoveTag(t) if t == "inference"));
    }

    #[test]
    fn test_diff_update_model_loaded() {
        let old = sample_capability_set();
        let mut new = old.clone();
        new.models[0].loaded = false;

        let ops = DiffEngine::diff(&old, &new);
        assert_eq!(ops.len(), 1);
        assert!(matches!(
            &ops[0],
            DiffOp::UpdateModel { model_id, loaded: Some(false), .. } if model_id == "llama-3.1-70b"
        ));
    }

    #[test]
    fn test_diff_add_model() {
        let old = sample_capability_set();
        let mut new = old.clone();
        new.models.push(
            ModelCapability::new("mistral-7b", "mistral")
                .with_parameters(7.0)
                .add_modality(Modality::Text),
        );

        let ops = DiffEngine::diff(&old, &new);
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], DiffOp::AddModel(m) if m.model_id == "mistral-7b"));
    }

    #[test]
    fn test_diff_update_memory() {
        let old = sample_capability_set();
        let mut new = old.clone();
        new.hardware.memory_mb = 131072;

        let ops = DiffEngine::diff(&old, &new);
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], DiffOp::UpdateMemory(131072)));
    }

    #[test]
    fn test_apply_diff() {
        let old = sample_capability_set();

        let diff = CapabilityDiff::new(
            1,
            1,
            2,
            vec![
                DiffOp::AddTag("training".into()),
                DiffOp::UpdateMemory(131072),
            ],
        );

        let new = DiffEngine::apply(&old, &diff, true).unwrap();

        assert!(new.has_tag("training"));
        assert_eq!(new.hardware.memory_mb, 131072);
    }

    #[test]
    fn test_apply_strict_error() {
        let caps = sample_capability_set();

        let diff = CapabilityDiff::new(1, 1, 2, vec![DiffOp::RemoveTag("nonexistent".into())]);

        let result = DiffEngine::apply(&caps, &diff, true);
        assert!(matches!(result, Err(DiffError::TagNotFound(_))));

        // Non-strict should succeed
        let result = DiffEngine::apply(&caps, &diff, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chain() {
        let diff1 = CapabilityDiff::new(1, 1, 2, vec![DiffOp::AddTag("a".into())]);
        let mut diff2 = CapabilityDiff::new(1, 2, 3, vec![DiffOp::AddTag("b".into())]);
        diff2.timestamp_ns = diff1.timestamp_ns + 1000;

        assert!(DiffEngine::validate_chain(&[diff1.clone(), diff2.clone()]));

        // Wrong base version
        let diff3 = CapabilityDiff::new(1, 1, 3, vec![DiffOp::AddTag("c".into())]);
        assert!(!DiffEngine::validate_chain(&[diff1.clone(), diff3]));
    }

    #[test]
    fn test_compact_diffs() {
        let base = sample_capability_set();

        let diff1 = CapabilityDiff::new(1, 1, 2, vec![DiffOp::AddTag("training".into())]);
        let diff2 = CapabilityDiff::new(1, 2, 3, vec![DiffOp::AddTag("distributed".into())]);
        let diff3 = CapabilityDiff::new(1, 3, 4, vec![DiffOp::UpdateMemory(131072)]);

        let compacted = DiffEngine::compact(&base, &[diff1, diff2, diff3]).unwrap();

        assert_eq!(compacted.base_version, 1);
        assert_eq!(compacted.new_version, 4);
        assert_eq!(compacted.ops.len(), 3); // 2 AddTag + 1 UpdateMemory
    }

    #[test]
    fn test_bandwidth_savings() {
        let caps = sample_capability_set();

        // Small diff
        let diff = CapabilityDiff::new(1, 1, 2, vec![DiffOp::AddTag("test".into())]);

        let (diff_size, full_size, savings) = DiffEngine::bandwidth_savings(&diff, &caps);

        assert!(diff_size < full_size);
        assert!(savings > 50.0); // Should save significant bandwidth
    }

    #[test]
    fn test_roundtrip_diff() {
        let old = sample_capability_set();
        let mut new = old.clone();

        // Make several changes
        new.tags.push("training".into());
        new.tags.retain(|t| t != "inference");
        new.models[0].loaded = false;
        new.models[0].tokens_per_sec = 100;
        new.hardware.memory_mb = 131072;
        new.models.push(
            ModelCapability::new("mistral-7b", "mistral")
                .with_parameters(7.0)
                .add_modality(Modality::Text),
        );

        // Generate diff
        let ops = DiffEngine::diff(&old, &new);
        let diff = CapabilityDiff::new(1, 1, 2, ops);

        // Apply diff
        let applied = DiffEngine::apply(&old, &diff, true).unwrap();

        // Verify
        assert!(applied.has_tag("training"));
        assert!(!applied.has_tag("inference"));
        assert_eq!(applied.hardware.memory_mb, 131072);
        assert_eq!(applied.models.len(), 2);
        assert!(
            !applied
                .models
                .iter()
                .find(|m| m.model_id == "llama-3.1-70b")
                .unwrap()
                .loaded
        );
    }

    #[test]
    fn test_diff_serialization() {
        let diff = CapabilityDiff::new(
            1,
            1,
            2,
            vec![DiffOp::AddTag("test".into()), DiffOp::UpdateMemory(65536)],
        );

        let bytes = diff.to_bytes();
        let parsed = CapabilityDiff::from_bytes(&bytes).unwrap();

        assert_eq!(diff.node_id, parsed.node_id);
        assert_eq!(diff.base_version, parsed.base_version);
        assert_eq!(diff.new_version, parsed.new_version);
        assert_eq!(diff.ops.len(), parsed.ops.len());
    }
}
