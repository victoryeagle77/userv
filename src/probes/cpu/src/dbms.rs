//! # Lib file for data base management system data module
//!
//! This module provides main functionality to set database parameters.

use core::core::{SQLiteKey, SQLiteOption, SQLiteType, SqlFieldDescriptor};

/// SQL table(s) available to create.
pub const TABLE_NAME: [&str; 4] = ["cpu_data", "cpu_core", "cpu_power", "cpu_temperature"];

/// # Returns
///
/// - Tuple of [`SqlFieldDescriptor`] set each table parameters values to insert in cpu_data database.
pub fn field_descriptor_info() -> Vec<SqlFieldDescriptor> {
    vec![
        SqlFieldDescriptor {
            field_name: "id",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::Primary,
            field_options: SQLiteOption::Autoincrement,
        },
        SqlFieldDescriptor {
            field_name: "timestamp",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "architecture",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "model",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "family",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "frequency",
            field_unit: Some("MHz"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "cores_physic",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "cores_logic",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

/// # Returns
///
/// - Tuple of [`SqlFieldDescriptor`] set each table parameters values to insert in cpu_core database.
pub fn field_descriptor_core() -> Vec<SqlFieldDescriptor> {
    vec![
        SqlFieldDescriptor {
            field_name: "id",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::Primary,
            field_options: SQLiteOption::Autoincrement,
        },
        SqlFieldDescriptor {
            field_name: "timestamp",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "core_name",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "usage",
            field_unit: Some("percent"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

/// # Returns
///
/// - Tuple of [`SqlFieldDescriptor`] set each table parameters values to insert in cpu_power database.
pub fn field_descriptor_power() -> Vec<SqlFieldDescriptor> {
    vec![
        SqlFieldDescriptor {
            field_name: "id",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::Primary,
            field_options: SQLiteOption::Autoincrement,
        },
        SqlFieldDescriptor {
            field_name: "timestamp",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "zone_name",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "power",
            field_unit: Some("W"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

/// # Returns
///
/// - Tuple of [`SqlFieldDescriptor`] set each table parameters values to insert in cpu_temperature database.
pub fn field_descriptor_temperature() -> Vec<SqlFieldDescriptor> {
    vec![
        SqlFieldDescriptor {
            field_name: "id",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::Primary,
            field_options: SQLiteOption::Autoincrement,
        },
        SqlFieldDescriptor {
            field_name: "timestamp",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "zone_name",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "temperature",
            field_unit: Some("°C"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}
