pub fn multiply(a: &String, b: &String, count: &mut u64) -> String {
  let value = format!(
      "\nLOAD {a}\nSTORE A\nLOAD {b}\nSTORE B\nJNS MUL\n"
  );
  *count += 1;
  value
}

pub fn add(a: &String, b: &String) -> String {
  format!("\nLOAD {a}\nADD {b}\n")
}

pub fn subtract(a: &String, b: &String) -> String {
  format!("\nLOAD {a}\nSUBT {b}\n")
}

pub fn divide(a: &String, b: &String, count: &mut u64) -> String {
  let value = format!(
      "\nLOAD {a}\nSTORE A\nLOAD {b}\nSTORE B\nJNS DIV\n"
  );
  *count += 1;
  value
}