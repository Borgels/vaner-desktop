pub fn validate_prepared_work_endpoint(endpoint: &str) -> Result<(), &'static str> {
    if !endpoint.starts_with('/') || endpoint.starts_with("//") {
        return Err("Invalid prepared work endpoint.");
    }
    if endpoint.contains("..") || !endpoint.starts_with("/work-products/") {
        return Err("Invalid prepared work endpoint.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_prepared_work_endpoint;

    #[test]
    fn allows_only_server_authoritative_work_product_paths() {
        assert!(validate_prepared_work_endpoint("/work-products/wp/export").is_ok());
        assert!(validate_prepared_work_endpoint("/work-products/wp/inspect").is_ok());
        assert!(validate_prepared_work_endpoint("/predictions/p/adopt").is_err());
        assert!(validate_prepared_work_endpoint("//evil.example/work-products/wp/export").is_err());
        assert!(validate_prepared_work_endpoint("/work-products/../config").is_err());
        assert!(
            validate_prepared_work_endpoint("https://example.com/work-products/wp/export").is_err()
        );
    }
}
