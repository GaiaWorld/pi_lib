#[ macro_export ] 
macro_rules! debug_println {
    ($($ arg: tt)*)=>(if cfg!(feature = "print"){ println!($($ arg)*);})
}

#[ macro_export ] 
macro_rules! debug_print {
    ($($ arg: tt)*)=>(if cfg!(feature = "print"){ print!($($ arg)*);})
}