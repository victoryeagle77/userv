//! # Lib file for data base management system data module
//!
//! This module provides main functionality to set database parameters.

use core::core::{SQLiteKey, SQLiteOption, SQLiteType, SqlFieldDescriptor};

/// SQL table(s) available to create.
pub const TABLE_NAME: &str = "board_data";

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
            field_name: "bios_date",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "bios_vendor",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "bios_version",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "board_name",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "board_serial",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::Unique,
        },
        SqlFieldDescriptor {
            field_name: "board_vendor",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "board_version",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test `field_descriptor` function structure
    #[test]
    fn test_field_descriptor_info() {
        let field = field_descriptor();
        let res = field
            .iter()
            .find(|f| f.field_name == "board_serial")
            .unwrap();
        assert_eq!(res.field_type, SQLiteType::Text);
        assert!(res.field_not_null);
        assert_eq!(res.field_key, SQLiteKey::None);
        assert_eq!(res.field_options, SQLiteOption::Unique);
    }
}
