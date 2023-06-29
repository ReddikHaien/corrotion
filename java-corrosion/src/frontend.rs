use std::array;

peg::parser!(
    grammar class_parser<'a>() for [u8]{

        rule u8() -> u8
            = x:[_] {x}

        rule u16() -> u16
            = v:$([_][_]) {unsafe{u16::from_be_bytes([*v.get_unchecked(0), *v.get_unchecked(1)])}}

        rule u32() -> u32
            = v:$([_][_][_][_]){ unsafe { u32::from_be_bytes([*v.get_unchecked(0), *v.get_unchecked(1), *v.get_unchecked(2), *v.get_unchecked(3)])} }
        
        rule magic()
            = quiet!{[0xca][0xfe][0xba][0xbe]}
            / expected!("magic number")

        rule version() -> (u16, u16)
        = minor:u16() major:u16() {(minor, major)}

        rule constant_pool(count: u16) -> ConstantPoolEntry<'input>
            = [7] index:u16() {ConstantPoolEntry::ClassInfo(index)}
            /

        rule class_file()
            = magic() 
              v:version()
              constant_pool_count: u16()
              constant_pool: constant_pool(constant_pool_count)

    }
);


pub enum ConstantPoolEntry<'a>{
    ClassInfo(u16)
}