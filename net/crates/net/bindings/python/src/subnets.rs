//! PyO3 surface for subnet configuration — mirror of
//! `bindings/node/src/subnets.rs`.
//!
//! `SubnetId` crosses the Python boundary as `list[int]` (1–4 entries,
//! each 0–255). `SubnetPolicy` crosses as
//! `{"rules": [{"tag_prefix": str, "level": int, "values": {str: int}}]}`.
//! Kept flat (no pyclass) to match the capability surface.

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use net::adapter::net::{SubnetId, SubnetPolicy, SubnetRule};

use crate::identity::identity_err;

fn u8_from_u32(value: u32, ctx: &str) -> PyResult<u8> {
    if value > 255 {
        return Err(identity_err(format!(
            "subnet: {} value {} exceeds u8 range",
            ctx, value
        )));
    }
    Ok(value as u8)
}

/// Convert a Python `list[int]` (1..=4 entries, each 0..=255) into
/// a core `SubnetId`.
pub fn subnet_id_from_py(levels: Vec<u32>) -> PyResult<SubnetId> {
    if levels.is_empty() {
        return Err(identity_err(
            "subnet: levels must have at least one entry".to_string(),
        ));
    }
    if levels.len() > 4 {
        return Err(identity_err(format!(
            "subnet: levels must have at most 4 entries, got {}",
            levels.len()
        )));
    }
    let mut bytes = [0u8; 4];
    for (i, raw) in levels.iter().enumerate() {
        bytes[i] = u8_from_u32(*raw, &format!("level {}", i))?;
    }
    Ok(SubnetId::new(&bytes[..levels.len()]))
}

fn subnet_rule_from_py(rule_dict: &Bound<'_, PyDict>) -> PyResult<SubnetRule> {
    let tag_prefix: String = rule_dict
        .get_item("tag_prefix")?
        .ok_or_else(|| identity_err("subnet: rule missing 'tag_prefix'"))?
        .extract()?;

    let raw_level: u32 = rule_dict
        .get_item("level")?
        .ok_or_else(|| identity_err("subnet: rule missing 'level'"))?
        .extract()?;
    let level = u8_from_u32(raw_level, "rule level")?;
    if level > 3 {
        return Err(identity_err(format!(
            "subnet: rule level must be 0..=3, got {}",
            level
        )));
    }

    let mut rule = SubnetRule::new(tag_prefix, level);
    if let Some(values_obj) = rule_dict.get_item("values")? {
        if !values_obj.is_none() {
            let values_dict = values_obj
                .cast_into::<PyDict>()
                .map_err(|_| PyTypeError::new_err("subnet: rule 'values' must be a dict"))?;
            for (key, val) in values_dict.iter() {
                let tag_value: String = key.extract()?;
                let raw: u32 = val.extract()?;
                let v = u8_from_u32(raw, &format!("rule value for {}", tag_value))?;
                rule = rule.map(tag_value, v);
            }
        }
    }
    Ok(rule)
}

/// Convert a Python `{"rules": [...]}` dict into a core
/// `SubnetPolicy`. Matches the NAPI helper's validation exactly —
/// level >= 4 is rejected before reaching `add_rule`, so the core's
/// debug-assert can't trip.
pub fn subnet_policy_from_py(d: &Bound<'_, PyDict>) -> PyResult<SubnetPolicy> {
    let mut policy = SubnetPolicy::new();
    let Some(rules_any) = d.get_item("rules")? else {
        return Ok(policy);
    };
    if rules_any.is_none() {
        return Ok(policy);
    }
    let rules_list = rules_any
        .cast_into::<PyList>()
        .map_err(|_| PyTypeError::new_err("subnet: 'rules' must be a list"))?;
    for item in rules_list.iter() {
        let rule_dict = item
            .cast_into::<PyDict>()
            .map_err(|_| PyTypeError::new_err("subnet: each rule must be a dict"))?;
        policy = policy.add_rule(subnet_rule_from_py(&rule_dict)?);
    }
    Ok(policy)
}
