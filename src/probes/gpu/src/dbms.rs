//! # Lib file for data base management system data module
//!
//! This module provides main functionality to set database parameters.

use core::core::{SQLiteKey, SQLiteOption, SQLiteType, SqlFieldDescriptor};

/// SQL table(s) available to create.
pub const TABLE_NAME: [&str; 2] = ["gpu_data", "gpu_process_data"];

/// # Returns
///
/// - A tuple of [`SqlFieldDescriptor`] describing each field of the `gpu_data` table to insert in database.
pub fn field_descriptor_gpu() -> Vec<SqlFieldDescriptor> {
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
            field_name: "gpu_architecture",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_bus_id",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_clock_graphic",
            field_unit: Some("MHz"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_clock_memory",
            field_unit: Some("MHz"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_clock_sm",
            field_unit: Some("MHz"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_clock_video",
            field_unit: Some("MHz"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_energy_consumption",
            field_unit: Some("mJ"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_name",
            field_unit: None,
            field_type: SQLiteType::Text,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_usage",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_temperature",
            field_unit: Some("°C"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_memory_free",
            field_unit: Some("B"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_memory_stat",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_memory_total",
            field_unit: Some("B"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_memory_usage",
            field_unit: None,
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_pci_data_sent",
            field_unit: Some("B_s"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_pci_data_received",
            field_unit: Some("B_s"),
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_power_consumption",
            field_unit: Some("mW"),
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "gpu_power_ratio",
            field_unit: None,
            field_type: SQLiteType::Real,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

/// # Returns
///
/// - A tuple of [`SqlFieldDescriptor`] describing each field of the `gpu_process_data` table to insert in database.
pub fn field_descriptor_process() -> Vec<SqlFieldDescriptor> {
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
            field_name: "gpu_bus_id",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "process_pid",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: true,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "process_decoding",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "process_encoding",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "process_memory",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
        SqlFieldDescriptor {
            field_name: "process_streaming_multiprocessor",
            field_unit: None,
            field_type: SQLiteType::Integer,
            field_not_null: false,
            field_key: SQLiteKey::None,
            field_options: SQLiteOption::None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test `field_descriptor_gpu` function structure
    #[test]
    fn test_field_descriptor_gpu() {
        let field = field_descriptor_gpu();
        let res = field.iter().find(|f| f.field_name == "gpu_bus_id").unwrap();
        assert_eq!(res.field_unit, None);
        assert_eq!(res.field_type, SQLiteType::Integer);
        assert!(res.field_not_null);
        assert_eq!(res.field_key, SQLiteKey::None);
        assert_eq!(res.field_options, SQLiteOption::None);
    }

    // Test `field_descriptor_process` function structure
    #[test]
    fn test_field_descriptor_process() {
        let field = field_descriptor_process();
        let res = field
            .iter()
            .find(|f| f.field_name == "process_pid")
            .unwrap();
        assert_eq!(res.field_unit, None);
        assert_eq!(res.field_type, SQLiteType::Integer);
        assert!(res.field_not_null);
        assert_eq!(res.field_key, SQLiteKey::None);
        assert_eq!(res.field_options, SQLiteOption::None);
    }
}
