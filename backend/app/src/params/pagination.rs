use serde::{Deserialize, Serialize};
use validator::Validate;

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    10
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Validate)]
pub struct PaginationParams {
    #[serde(
        default = "default_page",
        deserialize_with = "crate::serde::deserialize_number"
    )]
    #[validate(range(min = 1, message = "page must be at least 1"))]
    pub page: u64,
    #[serde(
        default = "default_per_page",
        deserialize_with = "crate::serde::deserialize_number"
    )]
    #[validate(range(min = 1, max = 100, message = "per_page must be between 1 and 100"))]
    pub per_page: u64,
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub items: Vec<T>,
}

impl<T> Page<T> {
    pub fn new(total: u64, page: u64, per_page: u64, items: Vec<T>) -> Self {
        Self {
            total,
            page,
            per_page,
            items,
        }
    }

    pub fn empty(per_page: u64) -> Self {
        Self {
            total: 0,
            page: 1,
            per_page,
            items: Vec::new(),
        }
    }

    pub fn from_pagination(pagination: PaginationParams, total: u64, items: Vec<T>) -> Self {
        Self {
            total,
            page: pagination.page,
            per_page: pagination.per_page,
            items,
        }
    }
}
