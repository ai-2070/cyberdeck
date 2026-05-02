// `#[napi]` exports to JS leave items "unused" from Rust's POV, so
// clippy's dead-code analysis doesn't apply to this module. Suppress
// at file scope.
#![allow(dead_code)]

//! NAPI surface for subnet configuration — the POJO shapes the TS
//! SDK passes through to the mesh, plus the conversions between them
//! and the core `SubnetId` / `SubnetRule` / `SubnetPolicy` types.
//!
//! `SubnetId` crosses the boundary as a variable-length `Vec<u8>`
//! (1–4 levels, each 0–255). Values beyond that are rejected on
//! conversion.

use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use net::adapter::net::{SubnetId, SubnetPolicy, SubnetRule};

// =========================================================================
// POJO types
// =========================================================================

#[napi(object)]
pub struct SubnetIdJs {
    /// 1–4 levels, each in `[0, 255]`. Example: `[3, 7, 2]` = level
    /// 0 bucket 3 / level 1 bucket 7 / level 2 bucket 2 / level 3 unset.
    pub levels: Vec<u32>,
}

#[napi(object)]
pub struct SubnetRuleJs {
    /// Tag prefix to match — e.g. `"region:"`. Matches at any
    /// position in the capability-tag list; the first matching tag
    /// per rule wins.
    pub tag_prefix: String,
    /// Hierarchy level (0–3) this rule fills.
    pub level: u32,
    /// `tag value → subnet byte` map. E.g. `{ "eu": 1, "us": 2 }`.
    /// Keys are the suffix after `tag_prefix`; values are the u8
    /// bucket index at `level`.
    pub values: HashMap<String, u32>,
}

#[napi(object)]
pub struct SubnetPolicyJs {
    pub rules: Vec<SubnetRuleJs>,
}

// =========================================================================
// Conversions
// =========================================================================

/// Validate + convert a JS `{ levels }` shape into a core `SubnetId`.
/// Rejects 0 or > 4 levels, or any byte > 255.
pub fn subnet_id_from_js(id: SubnetIdJs) -> Result<SubnetId> {
    if id.levels.is_empty() {
        return Err(Error::from_reason(
            "subnet: levels must have at least one entry".to_string(),
        ));
    }
    if id.levels.len() > 4 {
        return Err(Error::from_reason(format!(
            "subnet: levels must have at most 4 entries, got {}",
            id.levels.len()
        )));
    }
    let mut bytes = [0u8; 4];
    for (i, raw) in id.levels.iter().enumerate() {
        if *raw > 255 {
            return Err(Error::from_reason(format!(
                "subnet: level {} value {} exceeds u8 range",
                i, raw
            )));
        }
        bytes[i] = *raw as u8;
    }
    // `SubnetId::try_new` returns Result instead of panicking on
    // out-of-range input. The pre-validation above (length
    // check at lines 63-68) makes this branch unreachable, but
    // routing through the fallible API keeps the panic path off
    // the FFI surface entirely.
    SubnetId::try_new(&bytes[..id.levels.len()])
        .map_err(|e| Error::from_reason(format!("subnet: {}", e)))
}

fn parse_u8(value: u32, ctx: &str) -> Result<u8> {
    if value > 255 {
        return Err(Error::from_reason(format!(
            "subnet: {} value {} exceeds u8 range",
            ctx, value
        )));
    }
    Ok(value as u8)
}

fn subnet_rule_from_js(r: SubnetRuleJs) -> Result<SubnetRule> {
    let level = parse_u8(r.level, "rule level")?;
    if level > 3 {
        return Err(Error::from_reason(format!(
            "subnet: rule level must be 0..=3, got {}",
            level
        )));
    }
    let mut rule = SubnetRule::new(r.tag_prefix, level);
    for (tag_value, level_value) in r.values {
        let v = parse_u8(level_value, &format!("rule value for {}", tag_value))?;
        // Route through `try_map` so a `level_value == 0`
        // surfaces as a typed `SubnetError` (mapped to NAPI Error
        // here) rather than a native-side panic. This replaces the
        // earlier explicit `if v == 0` check at the NAPI boundary
        // with a single source-of-truth in the core.
        let tag_value_clone = tag_value.clone();
        rule = rule.try_map(tag_value, v).map_err(|e| {
            Error::from_reason(format!(
                "subnet: rule value for {:?}: {}",
                tag_value_clone, e
            ))
        })?;
    }
    Ok(rule)
}

/// Validate + convert a JS `SubnetPolicyJs` into a core
/// `SubnetPolicy`.
///
/// Routes through `try_add_rule` so any future loosening
/// of the per-rule pre-validation in `subnet_rule_from_js` still
/// surfaces an out-of-range `level` as a typed error rather than
/// a native panic.
pub fn subnet_policy_from_js(p: SubnetPolicyJs) -> Result<SubnetPolicy> {
    let mut policy = SubnetPolicy::new();
    for rule_js in p.rules {
        let rule = subnet_rule_from_js(rule_js)?;
        policy = policy
            .try_add_rule(rule)
            .map_err(|e| Error::from_reason(format!("subnet: {}", e)))?;
    }
    Ok(policy)
}
