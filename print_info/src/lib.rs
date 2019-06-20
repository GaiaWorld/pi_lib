#[ macro_export ] 
macro_rules! debug_println {
    ($($ arg: tt)*)=>(if cfg!(debug_assertions){ println!($($ arg)*);})
}

#[ macro_export ] 
macro_rules! debug_print {
    ($($ arg: tt)*)=>(if cfg!(debug_assertions){ print!($($ arg)*);})
}