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
      "CLEAR\nSTORE J\nSTORE POW\nLOAD {a}\nSTORE K\nCLEAR
  
OUTER_{count}, LOAD K
  SKIPCOND 800
  JUMP DONE_{count}
  LOAD ONE
  STORE POW
  LOAD {b}
  STORE J
  
INNER_{count}, LOAD J
  ADD J
  SUBT K
  SKIPCOND 000
  JUMP AFTIN_{count}
  LOAD J
  ADD J
  STORE J
  LOAD POW
  ADD POW
  STORE POW
  JUMP INNER_{count}
  
AFTIN_{count}, LOAD K
  SUBT J
  STORE K
  LOAD R
  ADD POW
  STORE R
  JUMP OUTER_{count}
  
DONE_{count}, LOAD K
  SKIPCOND 000
  JUMP DISP_{count}
  LOAD R
  SUBT ONE

DISP_{count}, LOAD R\n\n"
  );
  *count += 1;
  value
}