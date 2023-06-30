use std::{array, cell::{RefCell, RefMut}};

peg::parser!(
    grammar class_parser<'a>() for [u8]{

        rule u8() -> u8
            = x:[_] {x}

        rule u16() -> u16
            = v:$([_][_]) {unsafe{u16::from_be_bytes([*v.get_unchecked(0), *v.get_unchecked(1)])}}

        rule u32() -> u32
            = v:$([_][_][_][_]){ unsafe { u32::from_be_bytes([*v.get_unchecked(0), *v.get_unchecked(1), *v.get_unchecked(2), *v.get_unchecked(3)])} }
        rule u64() -> u64
            = v:$([_][_][_][_][_][_][_][_])
            { unsafe { 
                u64::from_be_bytes(
                    [*v.get_unchecked(0), *v.get_unchecked(1), *v.get_unchecked(2), *v.get_unchecked(3),
                     *v.get_unchecked(4), *v.get_unchecked(5), *v.get_unchecked(6), *v.get_unchecked(7)])} }
            
        rule f32() -> f32
            = v:u32() {f32::from_bits(v)}
        
        rule f64() -> f64
            = v:u64() {f64::from_bits(v)}
        
        rule slice(len: usize) -> &'input[u8]
            = v:$([_]*<{len}>) {v}

        rule magic()
            = quiet!{[0xca][0xfe][0xba][0xbe]}
            / expected!("magic number")

        rule version() -> (u16, u16)
        = minor:u16() major:u16() {(minor, major)}

        rule constant_pool_entry(count: &RefMut<u16>) -> ConstantPoolEntry<'input>
            = [1]  length:u16() bytes:slice(length as usize) {ConstantPoolEntry::Utf8Info(bytes)}
            / [3]  value:u32()                     {ConstantPoolEntry::IntegerInfo(value as i32)}
            / [4]  value:f32()                     {ConstantPoolEntry::FloatInfo(value)}
            / [5]  value:u64()                     {count.wrapping_sub(1); ConstantPoolEntry::LongInfo(value as i64)}
            / [6]  value:f64()                     {count.wrapping_sub(1); ConstantPoolEntry::DoubleInfo(value)}
            / [7]  index:u16()                     {ConstantPoolEntry::ClassInfo(index)}
            / [8]  index:u16()                     {ConstantPoolEntry::StringInfo(index)}
            / [9]  class:u16() name_and_type:u16() {ConstantPoolEntry::FieldRef { class, name_and_type }}
            / [10] class:u16() name_and_type:u16() {ConstantPoolEntry::MethodRef { class, name_and_type }}
            / [11] class:u16() name_and_type:u16() {ConstantPoolEntry::InterfaceMethodRef { class, name_and_type }}
            / [12] name:u16() descriptor:u16()     {ConstantPoolEntry::NameAndTypeInfo { name, descriptor }}

        rule constant_pool(count: RefMut<u16>) -> Vec<ConstantPoolEntry<'input>>
            = entries:constant_pool_entry(&count)*<{(*count) as usize}> {entries}

        rule class_file() -> ClassFile<'input>
            = magic() 
              version:version()
              constant_pool_count: (cp:u16() {RefCell::new(cp)})
              constant_pool: constant_pool(constant_pool_count.borrow_mut())
              {
                ClassFile{
                    constant_pool,
                    version
                }
              }
    }
);


pub enum ConstantPoolEntry<'a>{
    Utf8Info(&'a [u8]),
    ClassInfo(u16),
    FieldRef{
        class: u16,
        name_and_type: u16
    },
    MethodRef{
        class: u16,
        name_and_type: u16
    },
    InterfaceMethodRef{
        class: u16,
        name_and_type: u16
    },
    StringInfo(u16),
    IntegerInfo(i32),
    FloatInfo(f32),
    LongInfo(i64),
    DoubleInfo(f64),
    NameAndTypeInfo{
        name: u16,
        descriptor: u16
    }
}

struct ClassFile<'a>{
    pub version: (u16, u16),
    pub constant_pool: Vec<ConstantPoolEntry<'a>>
}