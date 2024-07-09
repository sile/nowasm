use crate::{
    components::{
        Code, Data, Elem, Export, FuncType, Function, Global, Import, Magic, MemType, TableType,
        TypeIdx, Version,
    },
    decode::Decode,
    reader::Reader,
    validate::ValidateError,
    DecodeError, VectorFactory,
};
use core::fmt::{Debug, Formatter};

pub struct Module<V: VectorFactory> {
    function_types: V::Vector<FuncType<V>>,
    imports: V::Vector<Import<V>>,
    functions: V::Vector<TypeIdx>, //TODO: Rename
    table_types: V::Vector<TableType>,
    memory_type: Option<MemType>,
    globals: V::Vector<Global<V>>,
    exports: V::Vector<Export<V>>,
    start_function: Option<Function>,
    elements: V::Vector<Elem<V>>,
    function_codes: V::Vector<Code<V>>,
    data_segments: V::Vector<Data<V>>,
}

impl<V: VectorFactory> Module<V> {
    pub fn decode(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut this = Self {
            function_types: V::create_vector(None),
            imports: V::create_vector(None),
            functions: V::create_vector(None),
            table_types: V::create_vector(None),
            memory_type: None,
            globals: V::create_vector(None),
            exports: V::create_vector(None),
            start_function: None,
            elements: V::create_vector(None),
            function_codes: V::create_vector(None),
            data_segments: V::create_vector(None),
        };
        let mut reader = Reader::new(wasm_bytes);

        // Preamble
        let _ = Magic::decode(&mut reader)?;
        let _ = Version::decode(&mut reader)?;

        // Sections
        this.decode_sections(&mut reader)?;

        Ok(this)
    }

    fn decode_sections(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        let mut last_section_id = SectionId::Custom;
        while !reader.is_empty() {
            let section_id = SectionId::decode(reader)?;
            let section_size = reader.read_u32()? as usize;
            let mut section_reader = Reader::new(reader.read(section_size)?);

            if section_id == SectionId::Custom {
                continue;
            }

            if section_id < last_section_id {
                return Err(DecodeError::InvalidSectionOrder {
                    current_section_id: section_id as u8,
                    last_section_id: last_section_id as u8,
                });
            }

            match section_id {
                SectionId::Custom => unreachable!(),
                SectionId::Type => {
                    self.function_types = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Import => {
                    self.imports = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Function => {
                    self.functions = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Table => {
                    self.table_types = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Memory => {
                    let value = section_reader.read_u32()? as usize;
                    if value > 1 {
                        return Err(DecodeError::InvalidMemoryCount { value });
                    }
                    if value == 1 {
                        let mem = MemType::decode(&mut section_reader)?;
                        self.memory_type = Some(mem);
                    }
                }
                SectionId::Global => {
                    self.globals = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Export => {
                    self.exports = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Start => {
                    self.start_function = Some(Decode::decode(&mut section_reader)?);
                }
                SectionId::Element => {
                    self.elements = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Code => {
                    self.function_codes = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SectionId::Data => {
                    self.data_segments = Decode::decode_vector::<V>(&mut section_reader)?;
                }
            }
            last_section_id = section_id;

            if !section_reader.is_empty() {
                return Err(DecodeError::InvalidSectionSize {
                    section_id: section_id as u8,
                    expected_size: section_size,
                    actual_size: section_reader.position(),
                });
            }
        }
        Ok(())
    }

    pub fn instantiate(&self) -> Result<(), ValidateError> {
        // TODO: validate
        Ok(())
    }

    pub fn function_types(&self) -> &[FuncType<V>] {
        &self.function_types
    }

    pub fn imports(&self) -> &[Import<V>] {
        &self.imports
    }

    pub fn functions(&self) -> &[TypeIdx] {
        &self.functions
    }

    pub fn table_types(&self) -> &[TableType] {
        &self.table_types
    }

    pub fn memory_type(&self) -> Option<MemType> {
        self.memory_type
    }

    pub fn globals(&self) -> &[Global<V>] {
        &self.globals
    }

    pub fn exports(&self) -> &[Export<V>] {
        &self.exports
    }

    pub fn start_function(&self) -> Option<Function> {
        self.start_function
    }

    pub fn elements(&self) -> &[Elem<V>] {
        &self.elements
    }

    pub fn function_codes(&self) -> &[Code<V>] {
        &self.function_codes
    }

    pub fn data_segments(&self) -> &[Data<V>] {
        &self.data_segments
    }
}

impl<V: VectorFactory> Debug for Module<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Module")
            .field("function_types", &self.function_types.as_ref())
            .field("imports", &self.imports.as_ref())
            .field("functions", &self.functions.as_ref())
            .field("table_types", &self.table_types.as_ref())
            .field("memory_type", &self.memory_type)
            .field("globals", &self.globals.as_ref())
            .field("exports", &self.exports.as_ref())
            .field("start_function", &self.start_function)
            .field("elements", &self.elements.as_ref())
            .field("function_codes", &self.function_codes.as_ref())
            .field("data_segments", &self.data_segments.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for Module<V> {
    fn clone(&self) -> Self {
        Self {
            function_types: V::clone_vector(&self.function_types),
            imports: V::clone_vector(&self.imports),
            functions: V::clone_vector(&self.functions),
            table_types: V::clone_vector(&self.table_types),
            memory_type: self.memory_type,
            globals: V::clone_vector(&self.globals),
            exports: V::clone_vector(&self.exports),
            start_function: self.start_function,
            elements: V::clone_vector(&self.elements),
            function_codes: V::clone_vector(&self.function_codes),
            data_segments: V::clone_vector(&self.data_segments),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SectionId {
    Custom = 0,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

impl SectionId {
    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0 => Ok(Self::Custom),
            1 => Ok(Self::Type),
            2 => Ok(Self::Import),
            3 => Ok(Self::Function),
            4 => Ok(Self::Table),
            5 => Ok(Self::Memory),
            6 => Ok(Self::Global),
            7 => Ok(Self::Export),
            8 => Ok(Self::Start),
            9 => Ok(Self::Element),
            10 => Ok(Self::Code),
            11 => Ok(Self::Data),
            value => Err(DecodeError::InvalidSectionId { value }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StdVectorFactory;

    fn decode(wasm: &[u8]) -> Module<StdVectorFactory> {
        Module::decode(wasm).expect("decode module")
    }

    #[test]
    fn decode_empty_module() {
        // (module)
        let input = [0, 97, 115, 109, 1, 0, 0, 0];
        decode(&input);
    }

    #[test]
    fn decode_add_two() {
        // (module
        //   (func (export "addTwo") (param i32 i32) (result i32)
        //     local.get 0
        //     local.get 1
        //     i32.add))
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 7, 1, 96, 2, 127, 127, 1, 127, 3, 2, 1, 0, 7, 10, 1, 6,
            97, 100, 100, 84, 119, 111, 0, 0, 10, 9, 1, 7, 0, 32, 0, 32, 1, 106, 11,
        ];
        let module = decode(&input);
        assert_eq!(1, module.exports().len());
        assert_eq!("addTwo", module.exports()[0].name.as_str());
    }
}
