/// Pagination parameters
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Pagination {
    pub fn to_sql(&self) -> String {
        match (self.limit, self.offset) {
            (Some(limit), Some(offset)) => format!(" LIMIT {} OFFSET {}", limit, offset),
            (Some(limit), None) => format!(" LIMIT {}", limit),
            (None, Some(offset)) => format!(" OFFSET {}", offset),
            (None, None) => String::new(),
        }
    }
}

/// Sorting parameters
pub struct Sorting {
    pub field: String,
    pub ascending: bool,
}

impl Sorting {
    pub fn to_sql(&self) -> String {
        let order = if self.ascending { "ASC" } else { "DESC" };
        format!(" ORDER BY {} {}", self.field, order)
    }
}

/// Filtering utility (simple key-value equality)
pub struct Filter {
    pub field: String,
    pub value: String,
}

impl Filter {
    pub fn to_sql(&self) -> String {
        format!(" WHERE {} = ?", self.field)
    }
}

/// Combine query parts for SELECT statements
pub fn build_select_query(
    base: &str,
    filter: Option<&Filter>,
    sorting: Option<&Sorting>,
    pagination: Option<&Pagination>,
) -> String {
    let mut query = String::from(base);
    if let Some(f) = filter {
        query.push_str(&f.to_sql());
    }
    if let Some(s) = sorting {
        query.push_str(&s.to_sql());
    }
    if let Some(p) = pagination {
        query.push_str(&p.to_sql());
    }
    query
} 