//! # Lib file for data base management system data module
//!
//! This module provides main functionality to set database parameters.

use core::core::{SQLiteKey, SQLiteOption, SQLiteType, SqlFieldDescriptor};

/// SQL table(s) available to create.
pub const TABLE_NAME: &str = "network_data";

/// # Returns
///
/// - A tuple of [`SqlFieldDescriptor`] describing each field of the table to insert in database.
pub fn field_descriptor() -> Vec<SqlFieldDescriptor> {
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
            field_name: "name",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "address_mac",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "network_type",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "received",
            field_unit: Some("MB"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "transmitted",
            field_unit: Some("MB"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "errors_received",
            field_unit: Some("MB"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "errors_transmitted",
            field_unit: Some("MB"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "packet_received",
            field_unit: Some("MB"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "packet_transmitted",
            field_unit: Some("MB"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "energy_consumed",
            field_unit: Some("W"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}
