//! Example: Extract Function refactoring operation.
//!
//! This example demonstrates how to use the ExtractFunction operation
//! to extract a block of code into a new function.
//!
//! Run with: cargo run --example extract_function

use refactor::prelude::*;

fn main() -> Result<()> {
    // Example source code with a block we want to extract
    let source_code = r#"
fn process_order(order: &Order) -> Result<(), Error> {
    // Validate order
    if order.items.is_empty() {
        return Err(Error::EmptyOrder);
    }
    if order.total < 0.0 {
        return Err(Error::InvalidTotal);
    }
    if order.customer_id.is_empty() {
        return Err(Error::MissingCustomer);
    }

    // Process the order
    save_order(order)?;
    notify_customer(order)?;

    Ok(())
}
"#;

    println!("=== Extract Function Example ===\n");
    println!("Original code:");
    println!("{}", source_code);

    // Create an extract function operation
    // We want to extract the validation logic (lines 3-11) into a separate function
    let extract = ExtractFunction::new("validate_order").with_visibility(Visibility::Private);

    println!("\nOperation: {}", extract.name());
    println!("Target: Extract lines 3-11 into 'validate_order' function");

    // In a real scenario, you would:
    // 1. Create a RefactoringContext with the file and selection range
    // 2. Validate the operation
    // 3. Preview the changes
    // 4. Apply the changes

    // For demonstration, show what the result would look like
    let expected_result = r#"
fn validate_order(order: &Order) -> Result<(), Error> {
    if order.items.is_empty() {
        return Err(Error::EmptyOrder);
    }
    if order.total < 0.0 {
        return Err(Error::InvalidTotal);
    }
    if order.customer_id.is_empty() {
        return Err(Error::MissingCustomer);
    }
    Ok(())
}

fn process_order(order: &Order) -> Result<(), Error> {
    // Validate order
    validate_order(order)?;

    // Process the order
    save_order(order)?;
    notify_customer(order)?;

    Ok(())
}
"#;

    println!("\nExpected result after extraction:");
    println!("{}", expected_result);

    // Demonstrate ExtractVariable as well
    println!("\n=== Extract Variable Example ===\n");

    let var_source = r#"
fn calculate_price(quantity: u32, unit_price: f64, discount: f64) -> f64 {
    quantity as f64 * unit_price * (1.0 - discount / 100.0)
}
"#;

    println!("Original code:");
    println!("{}", var_source);

    let extract_var = ExtractVariable::new("discounted_price").replace_all_occurrences();

    println!("Operation: {}", extract_var.name());
    println!("Target: Extract 'quantity as f64 * unit_price' into variable");

    let expected_var_result = r#"
fn calculate_price(quantity: u32, unit_price: f64, discount: f64) -> f64 {
    let discounted_price = quantity as f64 * unit_price;
    discounted_price * (1.0 - discount / 100.0)
}
"#;

    println!("\nExpected result:");
    println!("{}", expected_var_result);

    // Demonstrate the fluent API for RefactoringRunner
    println!("\n=== RefactoringRunner API ===\n");
    println!("You can use the RefactoringRunner for more control:");
    println!();
    println!(
        "{}",
        r#"
    let result = RefactoringRunner::new(ExtractFunction::new("my_fn"))
        .in_file("src/lib.rs")
        .at_range(Range::new(Position::new(10, 0), Position::new(20, 0)))
        .with_language(Rust)
        .dry_run()
        .execute()?;

    if result.is_valid {
        println!("Preview: {}", result.preview);
        // Apply: result.apply()?;
    }
"#
    );

    Ok(())
}
