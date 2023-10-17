pub fn multiply(a: &String, b: &String, count: &mut u64) -> String {
  let value = format!(
      "\nLOAD {a}\nSTORE COUNT\n\nLOOP_{count},	LOAD R
  ADD {b}
  STORE R
         
  LOAD COUNT
  SUBT ONE
  STORE COUNT
          
  SKIPCOND 400
  JUMP LOOP_{count}\n\nLOAD R\n"
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
      "CLEAR\nSTORE R\nSTORE C\nLOAD {a}\nSTORE A\nLOAD {b}\nSTORE B\n\nLOAD A\nSUBT B\nSKIPCOND 000
JNS SUB\nSKIPCOND 000\nDIV_{count},	JNS DIV\n\tSKIPCOND 000\n\tJUMP DIV_{count}\n\nLOAD R\n\n"
  );
  *count += 1;
  value
}