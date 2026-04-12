//! Capability Announcements (CAP-ANN) for Phase 4A.
//!
//! This module provides:
//! - `CapabilitySet` - Structured capability representation
//! - `CapabilityAnnouncement` - Versioned capability broadcast
//! - `CapabilityIndex` - High-performance capability indexing with inverted indexes
//! - `CapabilityFilter` - Query capabilities by various criteria
//!
//! # Performance Targets
//! - Index throughput: 100k+ announcements/s
//! - Query latency (single tag): < 100µs
//! - Query latency (complex filter): < 1ms
//! - Memory per 10k nodes: < 50MB

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

// ============================================================================
// Hardware Capabilities
// ============================================================================

/// GPU vendor enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum GpuVendor {
    /// Unrecognized or unspecified GPU vendor.
    #[default]
    Unknown = 0,
    /// NVIDIA Corporation.
    Nvidia = 1,
    /// Advanced Micro Devices (AMD).
    Amd = 2,
    /// Intel Corporation.
    Intel = 3,
    /// Apple Inc. (e.g., M-series integrated GPU).
    Apple = 4,
    /// Qualcomm (e.g., Adreno GPU).
    Qualcomm = 5,
}

impl From<u8> for GpuVendor {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Nvidia,
            2 => Self::Amd,
            3 => Self::Intel,
            4 => Self::Apple,
            5 => Self::Qualcomm,
            _ => Self::Unknown,
        }
    }
}

/// GPU information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU vendor
    pub vendor: GpuVendor,
    /// Model name (e.g., "RTX 4090", "M2 Ultra")
    pub model: String,
    /// VRAM in MB
    pub vram_mb: u32,
    /// Compute units / SMs
    pub compute_units: u16,
    /// Tensor cores (0 if none)
    pub tensor_cores: u16,
    /// FP16 TFLOPS (scaled by 10, e.g., 825 = 82.5 TFLOPS)
    pub fp16_tflops_x10: u16,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            vendor: GpuVendor::Unknown,
            model: String::new(),
            vram_mb: 0,
            compute_units: 0,
            tensor_cores: 0,
            fp16_tflops_x10: 0,
        }
    }
}

impl GpuInfo {
    /// Create new GPU info
    pub fn new(vendor: GpuVendor, model: impl Into<String>, vram_mb: u32) -> Self {
        Self {
            vendor,
            model: model.into(),
            vram_mb,
            ..Default::default()
        }
    }

    /// Set compute units
    pub fn with_compute_units(mut self, units: u16) -> Self {
        self.compute_units = units;
        self
    }

    /// Set tensor cores
    pub fn with_tensor_cores(mut self, cores: u16) -> Self {
        self.tensor_cores = cores;
        self
    }

    /// Set FP16 performance
    pub fn with_fp16_tflops(mut self, tflops: f32) -> Self {
        self.fp16_tflops_x10 = (tflops * 10.0) as u16;
        self
    }
}

/// Accelerator type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum AcceleratorType {
    /// Unrecognized or unspecified accelerator type.
    #[default]
    Unknown = 0,
    /// Tensor Processing Unit (e.g., Google TPU).
    Tpu = 1,
    /// Neural Processing Unit for on-device AI inference.
    Npu = 2,
    /// Field-Programmable Gate Array.
    Fpga = 3,
    /// Application-Specific Integrated Circuit.
    Asic = 4,
    /// Digital Signal Processor.
    Dsp = 5,
}

impl From<u8> for AcceleratorType {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Tpu,
            2 => Self::Npu,
            3 => Self::Fpga,
            4 => Self::Asic,
            5 => Self::Dsp,
            _ => Self::Unknown,
        }
    }
}

/// Accelerator information (TPU, NPU, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceleratorInfo {
    /// Accelerator type
    pub accel_type: AcceleratorType,
    /// Model/name
    pub model: String,
    /// Memory in MB (if applicable)
    pub memory_mb: u32,
    /// TOPS (tera operations per second, scaled by 10)
    pub tops_x10: u16,
}

impl AcceleratorInfo {
    /// Create new accelerator info
    pub fn new(accel_type: AcceleratorType, model: impl Into<String>) -> Self {
        Self {
            accel_type,
            model: model.into(),
            memory_mb: 0,
            tops_x10: 0,
        }
    }
}

/// Hardware capabilities
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    /// CPU cores
    pub cpu_cores: u16,
    /// CPU threads (if different from cores due to SMT)
    pub cpu_threads: u16,
    /// Total memory in MB
    pub memory_mb: u32,
    /// GPU info (if present)
    pub gpu: Option<GpuInfo>,
    /// Additional GPUs (for multi-GPU setups)
    pub additional_gpus: Vec<GpuInfo>,
    /// Storage in MB
    pub storage_mb: u64,
    /// Network bandwidth in Mbps
    pub network_mbps: u32,
    /// Accelerators (TPU, NPU, etc.)
    pub accelerators: Vec<AcceleratorInfo>,
}

impl HardwareCapabilities {
    /// Create new hardware capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Set CPU cores
    pub fn with_cpu(mut self, cores: u16, threads: u16) -> Self {
        self.cpu_cores = cores;
        self.cpu_threads = threads;
        self
    }

    /// Set memory
    pub fn with_memory(mut self, memory_mb: u32) -> Self {
        self.memory_mb = memory_mb;
        self
    }

    /// Set primary GPU
    pub fn with_gpu(mut self, gpu: GpuInfo) -> Self {
        self.gpu = Some(gpu);
        self
    }

    /// Add additional GPU
    pub fn add_gpu(mut self, gpu: GpuInfo) -> Self {
        self.additional_gpus.push(gpu);
        self
    }

    /// Set storage
    pub fn with_storage(mut self, storage_mb: u64) -> Self {
        self.storage_mb = storage_mb;
        self
    }

    /// Set network bandwidth
    pub fn with_network(mut self, network_mbps: u32) -> Self {
        self.network_mbps = network_mbps;
        self
    }

    /// Add accelerator
    pub fn add_accelerator(mut self, accel: AcceleratorInfo) -> Self {
        self.accelerators.push(accel);
        self
    }

    /// Total GPU count
    pub fn gpu_count(&self) -> usize {
        self.gpu.as_ref().map(|_| 1).unwrap_or(0) + self.additional_gpus.len()
    }

    /// Total VRAM across all GPUs
    pub fn total_vram_mb(&self) -> u32 {
        let primary = self.gpu.as_ref().map(|g| g.vram_mb).unwrap_or(0);
        let additional: u32 = self.additional_gpus.iter().map(|g| g.vram_mb).sum();
        primary + additional
    }

    /// Check if has any GPU
    pub fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    /// Get primary GPU vendor
    pub fn gpu_vendor(&self) -> Option<GpuVendor> {
        self.gpu.as_ref().map(|g| g.vendor)
    }
}

// ============================================================================
// Software Capabilities
// ============================================================================

/// Software/runtime capabilities
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SoftwareCapabilities {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Runtime versions (e.g., "python:3.11", "node:20")
    pub runtimes: Vec<(String, String)>,
    /// Installed frameworks (e.g., "pytorch:2.1", "tensorflow:2.15")
    pub frameworks: Vec<(String, String)>,
    /// CUDA version (if applicable)
    pub cuda_version: Option<String>,
    /// Driver versions
    pub drivers: Vec<(String, String)>,
}

impl SoftwareCapabilities {
    /// Create new software capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Set OS
    pub fn with_os(mut self, os: impl Into<String>, version: impl Into<String>) -> Self {
        self.os = os.into();
        self.os_version = version.into();
        self
    }

    /// Add runtime
    pub fn add_runtime(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.runtimes.push((name.into(), version.into()));
        self
    }

    /// Add framework
    pub fn add_framework(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.frameworks.push((name.into(), version.into()));
        self
    }

    /// Set CUDA version
    pub fn with_cuda(mut self, version: impl Into<String>) -> Self {
        self.cuda_version = Some(version.into());
        self
    }

    /// Check if has a specific runtime
    pub fn has_runtime(&self, name: &str) -> bool {
        self.runtimes.iter().any(|(n, _)| n == name)
    }

    /// Check if has a specific framework
    pub fn has_framework(&self, name: &str) -> bool {
        self.frameworks.iter().any(|(n, _)| n == name)
    }
}

// ============================================================================
// Model Capabilities
// ============================================================================

/// Modality support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Modality {
    /// Plain text input/output.
    Text = 0,
    /// Static image understanding or generation.
    Image = 1,
    /// Audio understanding or synthesis.
    Audio = 2,
    /// Video understanding or generation.
    Video = 3,
    /// Source code generation or analysis.
    Code = 4,
    /// Vector embedding production.
    Embedding = 5,
    /// Structured tool/function calling.
    ToolUse = 6,
}

impl From<u8> for Modality {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Text,
            1 => Self::Image,
            2 => Self::Audio,
            3 => Self::Video,
            4 => Self::Code,
            5 => Self::Embedding,
            6 => Self::ToolUse,
            _ => Self::Text,
        }
    }
}

/// Model capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapability {
    /// Unique model identifier (e.g., "llama-3.1-70b")
    pub model_id: String,
    /// Model family (e.g., "llama", "mistral", "claude")
    pub family: String,
    /// Parameter count (in billions, scaled by 10: 700 = 70B)
    pub parameters_b_x10: u32,
    /// Context length in tokens
    pub context_length: u32,
    /// Quantization (e.g., "fp16", "int8", "int4")
    pub quantization: Option<String>,
    /// Supported modalities
    pub modalities: Vec<Modality>,
    /// Estimated tokens per second (for this hardware)
    pub tokens_per_sec: u32,
    /// Whether model is currently loaded
    pub loaded: bool,
}

impl ModelCapability {
    /// Create new model capability
    pub fn new(model_id: impl Into<String>, family: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            family: family.into(),
            parameters_b_x10: 0,
            context_length: 0,
            quantization: None,
            modalities: vec![Modality::Text],
            tokens_per_sec: 0,
            loaded: false,
        }
    }

    /// Set parameter count in billions
    pub fn with_parameters(mut self, billions: f32) -> Self {
        self.parameters_b_x10 = (billions * 10.0) as u32;
        self
    }

    /// Set context length
    pub fn with_context_length(mut self, length: u32) -> Self {
        self.context_length = length;
        self
    }

    /// Set quantization
    pub fn with_quantization(mut self, quant: impl Into<String>) -> Self {
        self.quantization = Some(quant.into());
        self
    }

    /// Add modality
    pub fn add_modality(mut self, modality: Modality) -> Self {
        if !self.modalities.contains(&modality) {
            self.modalities.push(modality);
        }
        self
    }

    /// Set tokens per second
    pub fn with_tokens_per_sec(mut self, tps: u32) -> Self {
        self.tokens_per_sec = tps;
        self
    }

    /// Set loaded status
    pub fn with_loaded(mut self, loaded: bool) -> Self {
        self.loaded = loaded;
        self
    }

    /// Get parameter count as f32
    pub fn parameters(&self) -> f32 {
        self.parameters_b_x10 as f32 / 10.0
    }
}

// ============================================================================
// Tool Capabilities
// ============================================================================

/// Tool capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCapability {
    /// Unique tool identifier
    pub tool_id: String,
    /// Human-readable name
    pub name: String,
    /// Version
    pub version: String,
    /// Input schema (JSON Schema as string)
    pub input_schema: Option<String>,
    /// Output schema (JSON Schema as string)
    pub output_schema: Option<String>,
    /// Required capabilities/dependencies
    pub requires: Vec<String>,
    /// Estimated execution time in ms (for typical input)
    pub estimated_time_ms: u32,
    /// Whether tool is stateless
    pub stateless: bool,
}

impl ToolCapability {
    /// Create new tool capability
    pub fn new(tool_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            input_schema: None,
            output_schema: None,
            requires: Vec::new(),
            estimated_time_ms: 0,
            stateless: true,
        }
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set input schema
    pub fn with_input_schema(mut self, schema: impl Into<String>) -> Self {
        self.input_schema = Some(schema.into());
        self
    }

    /// Set output schema
    pub fn with_output_schema(mut self, schema: impl Into<String>) -> Self {
        self.output_schema = Some(schema.into());
        self
    }

    /// Add requirement
    pub fn requires(mut self, dep: impl Into<String>) -> Self {
        self.requires.push(dep.into());
        self
    }

    /// Set estimated time
    pub fn with_estimated_time(mut self, ms: u32) -> Self {
        self.estimated_time_ms = ms;
        self
    }

    /// Set stateless flag
    pub fn with_stateless(mut self, stateless: bool) -> Self {
        self.stateless = stateless;
        self
    }
}

// ============================================================================
// Resource Limits
// ============================================================================

/// Resource limits
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Maximum tokens per request
    pub max_tokens_per_request: u32,
    /// Rate limit (requests per minute)
    pub rate_limit_rpm: u32,
    /// Maximum batch size
    pub max_batch_size: u32,
    /// Maximum input size in bytes
    pub max_input_bytes: u32,
    /// Maximum output size in bytes
    pub max_output_bytes: u32,
}

impl ResourceLimits {
    /// Create new resource limits
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max concurrent requests
    pub fn with_max_concurrent(mut self, max: u32) -> Self {
        self.max_concurrent_requests = max;
        self
    }

    /// Set max tokens per request
    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens_per_request = max;
        self
    }

    /// Set rate limit
    pub fn with_rate_limit(mut self, rpm: u32) -> Self {
        self.rate_limit_rpm = rpm;
        self
    }

    /// Set max batch size
    pub fn with_max_batch(mut self, max: u32) -> Self {
        self.max_batch_size = max;
        self
    }
}

// ============================================================================
// Capability Set
// ============================================================================

/// Complete capability set for a node
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Hardware capabilities
    pub hardware: HardwareCapabilities,
    /// Software capabilities
    pub software: SoftwareCapabilities,
    /// Model capabilities
    pub models: Vec<ModelCapability>,
    /// Tool capabilities
    pub tools: Vec<ToolCapability>,
    /// Custom tags for filtering
    pub tags: Vec<String>,
    /// Resource limits
    pub limits: ResourceLimits,
}

impl CapabilitySet {
    /// Create empty capability set
    pub fn new() -> Self {
        Self::default()
    }

    /// Set hardware capabilities
    pub fn with_hardware(mut self, hardware: HardwareCapabilities) -> Self {
        self.hardware = hardware;
        self
    }

    /// Set software capabilities
    pub fn with_software(mut self, software: SoftwareCapabilities) -> Self {
        self.software = software;
        self
    }

    /// Add model capability
    pub fn add_model(mut self, model: ModelCapability) -> Self {
        self.models.push(model);
        self
    }

    /// Add tool capability
    pub fn add_tool(mut self, tool: ToolCapability) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add tag
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set resource limits
    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Check if has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if has a specific model
    pub fn has_model(&self, model_id: &str) -> bool {
        self.models.iter().any(|m| m.model_id == model_id)
    }

    /// Check if has a specific tool
    pub fn has_tool(&self, tool_id: &str) -> bool {
        self.tools.iter().any(|t| t.tool_id == tool_id)
    }

    /// Check if has GPU
    pub fn has_gpu(&self) -> bool {
        self.hardware.has_gpu()
    }

    /// Get all model IDs
    pub fn model_ids(&self) -> Vec<&str> {
        self.models.iter().map(|m| m.model_id.as_str()).collect()
    }

    /// Get all tool IDs
    pub fn tool_ids(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.tool_id.as_str()).collect()
    }

    /// Serialize to bytes (compact binary format)
    pub fn to_bytes(&self) -> Vec<u8> {
        // Use JSON for now (can optimize to binary later)
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

// ============================================================================
// Capability Announcement
// ============================================================================

/// Capability announcement message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAnnouncement {
    /// Announcing node ID
    pub node_id: u64,
    /// Monotonic version (for diffing)
    pub version: u64,
    /// Timestamp of announcement (nanoseconds since epoch)
    pub timestamp_ns: u64,
    /// TTL for this announcement in seconds
    pub ttl_secs: u32,
    /// Capability set
    pub capabilities: CapabilitySet,
    /// Optional Ed25519 signature (64 bytes, hex encoded for serde)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature64>,
}

/// 64-byte signature wrapper with serde support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature64(pub [u8; 64]);

impl Serialize for Signature64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as hex string for JSON compatibility
        if serializer.is_human_readable() {
            let hex = hex::encode(self.0);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Signature64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let hex_str = String::deserialize(deserializer)?;
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            if bytes.len() != 64 {
                return Err(serde::de::Error::custom("signature must be 64 bytes"));
            }
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&bytes);
            Ok(Signature64(arr))
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            if bytes.len() != 64 {
                return Err(serde::de::Error::custom("signature must be 64 bytes"));
            }
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&bytes);
            Ok(Signature64(arr))
        }
    }
}

impl CapabilityAnnouncement {
    /// Create new announcement
    pub fn new(node_id: u64, version: u64, capabilities: CapabilitySet) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        Self {
            node_id,
            version,
            timestamp_ns,
            ttl_secs: 300, // 5 minute default TTL
            capabilities,
            signature: None,
        }
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_secs: u32) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    /// Set signature
    pub fn with_signature(mut self, sig: [u8; 64]) -> Self {
        self.signature = Some(Signature64(sig));
        self
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let age_secs = (now_ns.saturating_sub(self.timestamp_ns)) / 1_000_000_000;
        age_secs > self.ttl_secs as u64
    }
}

// ============================================================================
// Capability Filter
// ============================================================================

/// Filter for querying capabilities
#[derive(Debug, Clone, Default)]
pub struct CapabilityFilter {
    /// Require specific tags (all must match)
    pub require_tags: Vec<String>,
    /// Require specific models (any must match)
    pub require_models: Vec<String>,
    /// Require specific tools (any must match)
    pub require_tools: Vec<String>,
    /// Minimum memory in MB
    pub min_memory_mb: Option<u32>,
    /// Require GPU
    pub require_gpu: bool,
    /// Specific GPU vendor
    pub gpu_vendor: Option<GpuVendor>,
    /// Minimum VRAM in MB
    pub min_vram_mb: Option<u32>,
    /// Minimum context length
    pub min_context_length: Option<u32>,
    /// Require specific modalities
    pub require_modalities: Vec<Modality>,
}

impl CapabilityFilter {
    /// Create empty filter (matches all)
    pub fn new() -> Self {
        Self::default()
    }

    /// Require tag
    pub fn require_tag(mut self, tag: impl Into<String>) -> Self {
        self.require_tags.push(tag.into());
        self
    }

    /// Require model
    pub fn require_model(mut self, model: impl Into<String>) -> Self {
        self.require_models.push(model.into());
        self
    }

    /// Require tool
    pub fn require_tool(mut self, tool: impl Into<String>) -> Self {
        self.require_tools.push(tool.into());
        self
    }

    /// Set minimum memory
    pub fn with_min_memory(mut self, mb: u32) -> Self {
        self.min_memory_mb = Some(mb);
        self
    }

    /// Require GPU
    pub fn require_gpu(mut self) -> Self {
        self.require_gpu = true;
        self
    }

    /// Require specific GPU vendor
    pub fn with_gpu_vendor(mut self, vendor: GpuVendor) -> Self {
        self.gpu_vendor = Some(vendor);
        self.require_gpu = true;
        self
    }

    /// Set minimum VRAM
    pub fn with_min_vram(mut self, mb: u32) -> Self {
        self.min_vram_mb = Some(mb);
        self.require_gpu = true;
        self
    }

    /// Set minimum context length
    pub fn with_min_context(mut self, length: u32) -> Self {
        self.min_context_length = Some(length);
        self
    }

    /// Require modality
    pub fn require_modality(mut self, modality: Modality) -> Self {
        self.require_modalities.push(modality);
        self
    }

    /// Check if a capability set matches this filter
    pub fn matches(&self, caps: &CapabilitySet) -> bool {
        // Check tags (all required tags must be present)
        for tag in &self.require_tags {
            if !caps.has_tag(tag) {
                return false;
            }
        }

        // Check models (any required model must be present)
        if !self.require_models.is_empty() {
            let has_model = self.require_models.iter().any(|m| caps.has_model(m));
            if !has_model {
                return false;
            }
        }

        // Check tools (any required tool must be present)
        if !self.require_tools.is_empty() {
            let has_tool = self.require_tools.iter().any(|t| caps.has_tool(t));
            if !has_tool {
                return false;
            }
        }

        // Check memory
        if let Some(min_mem) = self.min_memory_mb {
            if caps.hardware.memory_mb < min_mem {
                return false;
            }
        }

        // Check GPU
        if self.require_gpu && !caps.has_gpu() {
            return false;
        }

        // Check GPU vendor
        if let Some(vendor) = self.gpu_vendor {
            if caps.hardware.gpu_vendor() != Some(vendor) {
                return false;
            }
        }

        // Check VRAM
        if let Some(min_vram) = self.min_vram_mb {
            if caps.hardware.total_vram_mb() < min_vram {
                return false;
            }
        }

        // Check context length
        if let Some(min_ctx) = self.min_context_length {
            let has_sufficient = caps.models.iter().any(|m| m.context_length >= min_ctx);
            if !has_sufficient {
                return false;
            }
        }

        // Check modalities
        for modality in &self.require_modalities {
            let has_modality = caps.models.iter().any(|m| m.modalities.contains(modality));
            if !has_modality {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// Capability Requirement (for load balancing)
// ============================================================================

/// Capability requirement with scoring
#[derive(Debug, Clone, Default)]
pub struct CapabilityRequirement {
    /// Base filter
    pub filter: CapabilityFilter,
    /// Prefer more memory (weight 0.0-1.0)
    pub prefer_more_memory: f32,
    /// Prefer more VRAM (weight 0.0-1.0)
    pub prefer_more_vram: f32,
    /// Prefer faster tokens/sec (weight 0.0-1.0)
    pub prefer_faster_inference: f32,
    /// Prefer loaded models (weight 0.0-1.0)
    pub prefer_loaded_models: f32,
}

impl CapabilityRequirement {
    /// Create from filter
    pub fn from_filter(filter: CapabilityFilter) -> Self {
        Self {
            filter,
            ..Default::default()
        }
    }

    /// Set memory preference weight
    pub fn prefer_memory(mut self, weight: f32) -> Self {
        self.prefer_more_memory = weight.clamp(0.0, 1.0);
        self
    }

    /// Set VRAM preference weight
    pub fn prefer_vram(mut self, weight: f32) -> Self {
        self.prefer_more_vram = weight.clamp(0.0, 1.0);
        self
    }

    /// Set inference speed preference
    pub fn prefer_speed(mut self, weight: f32) -> Self {
        self.prefer_faster_inference = weight.clamp(0.0, 1.0);
        self
    }

    /// Set loaded model preference
    pub fn prefer_loaded(mut self, weight: f32) -> Self {
        self.prefer_loaded_models = weight.clamp(0.0, 1.0);
        self
    }

    /// Score a capability set (higher is better)
    pub fn score(&self, caps: &CapabilitySet) -> f32 {
        if !self.filter.matches(caps) {
            return 0.0;
        }

        let mut score = 1.0;

        // Memory score (normalized to 256GB)
        if self.prefer_more_memory > 0.0 {
            let mem_score = (caps.hardware.memory_mb as f32 / 262144.0).min(1.0);
            score += self.prefer_more_memory * mem_score;
        }

        // VRAM score (normalized to 80GB)
        if self.prefer_more_vram > 0.0 {
            let vram_score = (caps.hardware.total_vram_mb() as f32 / 81920.0).min(1.0);
            score += self.prefer_more_vram * vram_score;
        }

        // Inference speed score (normalized to 1000 tok/s)
        if self.prefer_faster_inference > 0.0 {
            let max_tps: u32 = caps
                .models
                .iter()
                .map(|m| m.tokens_per_sec)
                .max()
                .unwrap_or(0);
            let speed_score = (max_tps as f32 / 1000.0).min(1.0);
            score += self.prefer_faster_inference * speed_score;
        }

        // Loaded model score
        if self.prefer_loaded_models > 0.0 {
            let loaded_count = caps.models.iter().filter(|m| m.loaded).count();
            let loaded_ratio = if caps.models.is_empty() {
                0.0
            } else {
                loaded_count as f32 / caps.models.len() as f32
            };
            score += self.prefer_loaded_models * loaded_ratio;
        }

        score
    }
}

// ============================================================================
// Capability Index
// ============================================================================

/// Indexed node entry
#[derive(Debug, Clone)]
pub struct IndexedNode {
    /// Node ID
    pub node_id: u64,
    /// Capability set
    pub capabilities: CapabilitySet,
    /// Version
    pub version: u64,
    /// When indexed
    pub indexed_at: Instant,
    /// TTL
    pub ttl: Duration,
}

/// High-performance capability index with inverted indexes
pub struct CapabilityIndex {
    /// Node ID -> indexed node
    nodes: DashMap<u64, IndexedNode>,
    /// Inverted index: tag -> set of node IDs
    by_tag: DashMap<String, HashSet<u64>>,
    /// Inverted index: model ID -> set of node IDs
    by_model: DashMap<String, HashSet<u64>>,
    /// Inverted index: tool ID -> set of node IDs
    by_tool: DashMap<String, HashSet<u64>>,
    /// Inverted index: GPU vendor -> set of node IDs
    by_gpu_vendor: DashMap<GpuVendor, HashSet<u64>>,
    /// Inverted index: has GPU -> set of node IDs
    gpu_nodes: DashMap<bool, HashSet<u64>>,
    /// Version tracking
    versions: DashMap<u64, u64>,
    /// Stats
    index_count: AtomicU64,
    query_count: AtomicU64,
}

impl CapabilityIndex {
    /// Create new capability index
    pub fn new() -> Self {
        Self {
            nodes: DashMap::new(),
            by_tag: DashMap::new(),
            by_model: DashMap::new(),
            by_tool: DashMap::new(),
            by_gpu_vendor: DashMap::new(),
            gpu_nodes: DashMap::new(),
            versions: DashMap::new(),
            index_count: AtomicU64::new(0),
            query_count: AtomicU64::new(0),
        }
    }

    /// Index a capability announcement
    pub fn index(&self, ann: CapabilityAnnouncement) {
        let node_id = ann.node_id;

        // Check version - only index if newer
        let should_index = self
            .versions
            .get(&node_id)
            .map(|v| ann.version > *v)
            .unwrap_or(true);

        if !should_index {
            return;
        }

        // Remove old entries from inverted indexes
        if let Some(old) = self.nodes.get(&node_id) {
            self.remove_from_indexes(node_id, &old.capabilities);
        }

        // Update version
        self.versions.insert(node_id, ann.version);

        // Add to inverted indexes
        self.add_to_indexes(node_id, &ann.capabilities);

        // Store node
        let indexed = IndexedNode {
            node_id,
            capabilities: ann.capabilities,
            version: ann.version,
            indexed_at: Instant::now(),
            ttl: Duration::from_secs(ann.ttl_secs as u64),
        };
        self.nodes.insert(node_id, indexed);

        self.index_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Remove node from index
    pub fn remove(&self, node_id: u64) {
        if let Some((_, node)) = self.nodes.remove(&node_id) {
            self.remove_from_indexes(node_id, &node.capabilities);
            self.versions.remove(&node_id);
        }
    }

    /// Add node to inverted indexes
    fn add_to_indexes(&self, node_id: u64, caps: &CapabilitySet) {
        // Tags
        for tag in &caps.tags {
            self.by_tag.entry(tag.clone()).or_default().insert(node_id);
        }

        // Models
        for model in &caps.models {
            self.by_model
                .entry(model.model_id.clone())
                .or_default()
                .insert(node_id);
        }

        // Tools
        for tool in &caps.tools {
            self.by_tool
                .entry(tool.tool_id.clone())
                .or_default()
                .insert(node_id);
        }

        // GPU
        let has_gpu = caps.has_gpu();
        self.gpu_nodes.entry(has_gpu).or_default().insert(node_id);

        if let Some(vendor) = caps.hardware.gpu_vendor() {
            self.by_gpu_vendor
                .entry(vendor)
                .or_default()
                .insert(node_id);
        }
    }

    /// Remove node from inverted indexes
    fn remove_from_indexes(&self, node_id: u64, caps: &CapabilitySet) {
        // Tags
        for tag in &caps.tags {
            if let Some(mut set) = self.by_tag.get_mut(tag) {
                set.remove(&node_id);
            }
        }

        // Models
        for model in &caps.models {
            if let Some(mut set) = self.by_model.get_mut(&model.model_id) {
                set.remove(&node_id);
            }
        }

        // Tools
        for tool in &caps.tools {
            if let Some(mut set) = self.by_tool.get_mut(&tool.tool_id) {
                set.remove(&node_id);
            }
        }

        // GPU
        let has_gpu = caps.has_gpu();
        if let Some(mut set) = self.gpu_nodes.get_mut(&has_gpu) {
            set.remove(&node_id);
        }

        if let Some(vendor) = caps.hardware.gpu_vendor() {
            if let Some(mut set) = self.by_gpu_vendor.get_mut(&vendor) {
                set.remove(&node_id);
            }
        }
    }

    /// Query nodes by filter
    pub fn query(&self, filter: &CapabilityFilter) -> Vec<u64> {
        self.query_count.fetch_add(1, Ordering::Relaxed);

        // Start with candidate set
        let mut candidates: Option<HashSet<u64>> = None;

        // Use inverted indexes to narrow down candidates

        // GPU filter (most selective often)
        if filter.require_gpu {
            if let Some(gpu_nodes) = self.gpu_nodes.get(&true) {
                candidates = Some(gpu_nodes.clone());
            } else {
                return Vec::new();
            }
        }

        // GPU vendor filter
        if let Some(vendor) = filter.gpu_vendor {
            if let Some(vendor_nodes) = self.by_gpu_vendor.get(&vendor) {
                candidates = Some(match candidates {
                    Some(c) => c.intersection(&vendor_nodes).copied().collect(),
                    None => vendor_nodes.clone(),
                });
            } else {
                return Vec::new();
            }
        }

        // Tag filter (all required)
        for tag in &filter.require_tags {
            if let Some(tag_nodes) = self.by_tag.get(tag) {
                candidates = Some(match candidates {
                    Some(c) => c.intersection(&tag_nodes).copied().collect(),
                    None => tag_nodes.clone(),
                });
            } else {
                return Vec::new();
            }
        }

        // Model filter (any required)
        if !filter.require_models.is_empty() {
            let mut model_candidates = HashSet::new();
            for model in &filter.require_models {
                if let Some(model_nodes) = self.by_model.get(model) {
                    model_candidates.extend(model_nodes.iter());
                }
            }
            if model_candidates.is_empty() {
                return Vec::new();
            }
            candidates = Some(match candidates {
                Some(c) => c.intersection(&model_candidates).copied().collect(),
                None => model_candidates,
            });
        }

        // Tool filter (any required)
        if !filter.require_tools.is_empty() {
            let mut tool_candidates = HashSet::new();
            for tool in &filter.require_tools {
                if let Some(tool_nodes) = self.by_tool.get(tool) {
                    tool_candidates.extend(tool_nodes.iter());
                }
            }
            if tool_candidates.is_empty() {
                return Vec::new();
            }
            candidates = Some(match candidates {
                Some(c) => c.intersection(&tool_candidates).copied().collect(),
                None => tool_candidates,
            });
        }

        // If no indexed filters applied, start with all nodes
        let candidates =
            candidates.unwrap_or_else(|| self.nodes.iter().map(|r| *r.key()).collect());

        // Apply remaining filters that need full capability check
        candidates
            .into_iter()
            .filter(|&node_id| {
                self.nodes
                    .get(&node_id)
                    .map(|n| filter.matches(&n.capabilities))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Find best matching node using requirements
    pub fn find_best(&self, req: &CapabilityRequirement) -> Option<u64> {
        let candidates = self.query(&req.filter);

        candidates
            .into_iter()
            .filter_map(|node_id| {
                self.nodes
                    .get(&node_id)
                    .map(|n| (node_id, req.score(&n.capabilities)))
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(node_id, _)| node_id)
    }

    /// Get node capabilities
    pub fn get(&self, node_id: u64) -> Option<CapabilitySet> {
        self.nodes.get(&node_id).map(|n| n.capabilities.clone())
    }

    /// Get all node IDs
    pub fn all_nodes(&self) -> Vec<u64> {
        self.nodes.iter().map(|r| *r.key()).collect()
    }

    /// Get node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Garbage collect expired entries
    pub fn gc(&self) -> usize {
        let now = Instant::now();
        let mut removed = 0;

        let expired: Vec<u64> = self
            .nodes
            .iter()
            .filter(|r| now.duration_since(r.indexed_at) > r.ttl)
            .map(|r| *r.key())
            .collect();

        for node_id in expired {
            self.remove(node_id);
            removed += 1;
        }

        removed
    }

    /// Get statistics
    pub fn stats(&self) -> CapabilityIndexStats {
        CapabilityIndexStats {
            node_count: self.nodes.len(),
            tag_count: self.by_tag.len(),
            model_count: self.by_model.len(),
            tool_count: self.by_tool.len(),
            total_indexed: self.index_count.load(Ordering::Relaxed),
            total_queries: self.query_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for CapabilityIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability index statistics
#[derive(Debug, Clone, Default)]
pub struct CapabilityIndexStats {
    /// Number of indexed nodes
    pub node_count: usize,
    /// Number of unique tags
    pub tag_count: usize,
    /// Number of unique models
    pub model_count: usize,
    /// Number of unique tools
    pub tool_count: usize,
    /// Total announcements indexed
    pub total_indexed: u64,
    /// Total queries processed
    pub total_queries: u64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_capability_set() -> CapabilitySet {
        let gpu = GpuInfo::new(GpuVendor::Nvidia, "RTX 4090", 24576)
            .with_compute_units(128)
            .with_tensor_cores(512)
            .with_fp16_tflops(82.5);

        let hardware = HardwareCapabilities::new()
            .with_cpu(16, 32)
            .with_memory(65536)
            .with_gpu(gpu)
            .with_storage(2_000_000)
            .with_network(10000);

        let software = SoftwareCapabilities::new()
            .with_os("linux", "6.1")
            .add_runtime("python", "3.11")
            .add_framework("pytorch", "2.1")
            .with_cuda("12.1");

        let model = ModelCapability::new("llama-3.1-70b", "llama")
            .with_parameters(70.0)
            .with_context_length(128000)
            .with_quantization("fp16")
            .add_modality(Modality::Text)
            .add_modality(Modality::Code)
            .with_tokens_per_sec(50)
            .with_loaded(true);

        let tool = ToolCapability::new("python_repl", "Python REPL")
            .with_version("1.0.0")
            .with_estimated_time(100);

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
    fn test_capability_set_creation() {
        let caps = sample_capability_set();
        assert!(caps.has_gpu());
        assert!(caps.has_tag("inference"));
        assert!(caps.has_model("llama-3.1-70b"));
        assert!(caps.has_tool("python_repl"));
        assert_eq!(caps.hardware.memory_mb, 65536);
    }

    #[test]
    fn test_capability_set_serialization() {
        let caps = sample_capability_set();
        let bytes = caps.to_bytes();
        let parsed = CapabilitySet::from_bytes(&bytes).unwrap();

        assert_eq!(caps.hardware.memory_mb, parsed.hardware.memory_mb);
        assert_eq!(caps.tags, parsed.tags);
        assert_eq!(caps.models.len(), parsed.models.len());
    }

    #[test]
    fn test_capability_filter_matches() {
        let caps = sample_capability_set();

        // Tag filter
        let filter = CapabilityFilter::new().require_tag("inference");
        assert!(filter.matches(&caps));

        let filter = CapabilityFilter::new().require_tag("training");
        assert!(!filter.matches(&caps));

        // GPU filter
        let filter = CapabilityFilter::new().require_gpu();
        assert!(filter.matches(&caps));

        let filter = CapabilityFilter::new().with_gpu_vendor(GpuVendor::Nvidia);
        assert!(filter.matches(&caps));

        let filter = CapabilityFilter::new().with_gpu_vendor(GpuVendor::Amd);
        assert!(!filter.matches(&caps));

        // Memory filter
        let filter = CapabilityFilter::new().with_min_memory(32768);
        assert!(filter.matches(&caps));

        let filter = CapabilityFilter::new().with_min_memory(131072);
        assert!(!filter.matches(&caps));

        // Model filter
        let filter = CapabilityFilter::new().require_model("llama-3.1-70b");
        assert!(filter.matches(&caps));

        let filter = CapabilityFilter::new().require_model("gpt-4");
        assert!(!filter.matches(&caps));
    }

    #[test]
    fn test_capability_index() {
        let index = CapabilityIndex::new();

        // Index some nodes
        for i in 0..100 {
            let mut caps = sample_capability_set();
            if i % 2 == 0 {
                caps.tags.push("even".into());
            }
            if i % 3 == 0 {
                caps.tags.push("divisible_by_3".into());
            }

            let ann = CapabilityAnnouncement::new(i, 1, caps);
            index.index(ann);
        }

        assert_eq!(index.len(), 100);

        // Query by tag
        let filter = CapabilityFilter::new().require_tag("even");
        let results = index.query(&filter);
        assert_eq!(results.len(), 50);

        // Query by multiple tags
        let filter = CapabilityFilter::new()
            .require_tag("even")
            .require_tag("divisible_by_3");
        let results = index.query(&filter);
        // Nodes divisible by 6: 0, 6, 12, 18, 24, 30, 36, 42, 48, 54, 60, 66, 72, 78, 84, 90, 96
        assert_eq!(results.len(), 17);

        // Query by GPU
        let filter = CapabilityFilter::new().require_gpu();
        let results = index.query(&filter);
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_capability_requirement_scoring() {
        let caps = sample_capability_set();

        let req = CapabilityRequirement::from_filter(CapabilityFilter::new().require_gpu())
            .prefer_memory(0.5)
            .prefer_vram(0.5)
            .prefer_speed(0.5);

        let score = req.score(&caps);
        assert!(score > 1.0); // Base score + preferences
    }

    #[test]
    fn test_capability_index_version_handling() {
        let index = CapabilityIndex::new();

        // Index version 1
        let caps_v1 = CapabilitySet::new().add_tag("v1");
        let ann_v1 = CapabilityAnnouncement::new(1, 1, caps_v1);
        index.index(ann_v1);

        // Query should find v1 tag
        let filter = CapabilityFilter::new().require_tag("v1");
        assert_eq!(index.query(&filter).len(), 1);

        // Index version 2 (should replace v1)
        let caps_v2 = CapabilitySet::new().add_tag("v2");
        let ann_v2 = CapabilityAnnouncement::new(1, 2, caps_v2);
        index.index(ann_v2);

        // v1 tag should be gone, v2 should be present
        let filter = CapabilityFilter::new().require_tag("v1");
        assert_eq!(index.query(&filter).len(), 0);

        let filter = CapabilityFilter::new().require_tag("v2");
        assert_eq!(index.query(&filter).len(), 1);

        // Older version should be ignored
        let caps_old = CapabilitySet::new().add_tag("old");
        let ann_old = CapabilityAnnouncement::new(1, 1, caps_old);
        index.index(ann_old);

        // v2 should still be present
        let filter = CapabilityFilter::new().require_tag("v2");
        assert_eq!(index.query(&filter).len(), 1);
    }

    #[test]
    fn test_capability_announcement_expiry() {
        let caps = sample_capability_set();
        let mut ann = CapabilityAnnouncement::new(1, 1, caps);

        // Fresh announcement should not be expired
        assert!(!ann.is_expired());

        // Set timestamp to the past
        ann.timestamp_ns = 0;
        ann.ttl_secs = 1;

        // Should be expired now
        assert!(ann.is_expired());
    }
}
