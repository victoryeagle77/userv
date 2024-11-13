//! # Lib file for data base management system data module
//!
//! This module provides main functionality to set database parameters.

use core::core::{SQLiteKey, SQLiteOption, SQLiteType, SqlFieldDescriptor};

/// SQL table(s) available to create.
pub const TABLE_NAME: [&str; 2] = ["memory_data", "memory_modules"];

/// # Returns
///
/// - Tuple of [`SqlFieldDescriptor`] set each table parameters values to insert in memory_data database.
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
            field_unit: Some("MB_s"),
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "bandwidth_read",
            field_unit: Some("MB_s"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "bandwidth_write",
            field_unit: None,
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "ram_total",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "ram_used",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "ram_free",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "ram_available",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "ram_power_consumption",
            field_unit: Some("W"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "swap_total",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "swap_used",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "swap_free",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

/// # Returns
///
/// - Tuple of [`SqlFieldDescriptor`] set each table parameters values to insert in memory_modules database.
pub fn field_descriptor_device() -> Vec<SqlFieldDescriptor> {
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
            field_name: "device_id",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::Unique,
        },
        SqlFieldDescriptor {
            field_name: "ram_type",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "size",
            field_unit: Some("MB"),
            field_type: SQLiteType::Integer,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "speed",
            field_unit: Some("mt_s"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "voltage",
            field_unit: Some("mV"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test `field_descriptor_info` function structure
    #[test]
    fn test_field_descriptor_info() {
        let field = field_descriptor_info();
        let res = field.iter().find(|f| f.field_name == "ram_total").unwrap();
        assert_eq!(res.field_unit, Some("MB"));
        assert_eq!(res.field_type, SQLiteType::Integer);
        assert!(res.field_not_null);
        assert_eq!(res.field_key, SQLiteKey::None);
        assert_eq!(res.field_options, SQLiteOption::None);
    }

    // Test `field_descriptor_device` function structure
    #[test]
    fn test_field_descriptor_device() {
        let field = field_descriptor_device();
        let res = field.iter().find(|f| f.field_name == "voltage").unwrap();
        assert_eq!(res.field_unit, Some("mV"));
        assert_eq!(res.field_type, SQLiteType::Real);
        assert_eq!(res.field_key, SQLiteKey::None);
        assert_eq!(res.field_options, SQLiteOption::None);
    }
}
