use crate::{
    reader::Reader,
    symbols::{Code, Data, Elem, Export, FuncType, Global, Import, Magic, SectionId, Version},
    vectors::NullVectors,
    DecodeError,
};

// TODO: s/ModuleSpec/Module<V: Vectors>/
#[derive(Debug, Clone)]
pub struct ModuleSpec {
    pub func_types: usize,
    pub imports: usize,
    pub bytes: usize,
    pub idxs: usize,
    pub table_types: usize,
    pub val_types: usize,
    pub globals: usize,
    pub instrs: usize,
    pub exports: usize,
    pub elements: usize,
    pub codes: usize,
    pub locals: usize,
    pub data_segments: usize,
}

impl ModuleSpec {
    pub fn inspect(wasm_bytes: &[u8]) -> Result<ModuleSpec, DecodeError> {
        let mut reader = Reader::new(wasm_bytes);
        let mut this = Self {
            func_types: 0,
            imports: 0,
            bytes: 0,
            idxs: 0,
            table_types: 0,
            val_types: 0,
            globals: 0,
            instrs: 0,
            exports: 0,
            elements: 0,
            codes: 0,
            locals: 0,
            data_segments: 0,
        };
        this.handle_module(&mut reader)?;
        Ok(this)
    }

    fn handle_module(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        // Preamble
        let _ = Magic::decode(reader)?;
        let _ = Version::decode(reader)?;

        // Sections
        let mut last_section_id = SectionId::Custom;
        while !reader.is_empty() {
            let section_id = SectionId::decode(reader)?;
            if section_id != SectionId::Custom {
                if section_id <= last_section_id {
                    return Err(DecodeError::OutOfOrderSectionId {
                        last: last_section_id,
                        current: section_id,
                    });
                }
                last_section_id = section_id;
            }

            let section_size = reader.read_u32()? as usize;
            let mut section_reader = Reader::new(reader.read(section_size)?);

            match section_id {
                SectionId::Custom => {}
                SectionId::Type => self.handle_type_section(&mut section_reader)?,
                SectionId::Import => self.handle_import_section(&mut section_reader)?,
                SectionId::Function => self.handle_function_section(&mut section_reader)?,
                SectionId::Table => self.handle_table_section(&mut section_reader)?,
                SectionId::Memory => self.handle_memory_section(&mut section_reader)?,
                SectionId::Global => self.handle_global_section(&mut section_reader)?,
                SectionId::Export => self.handle_export_section(&mut section_reader)?,
                SectionId::Start => self.handle_start_section(&mut section_reader)?,
                SectionId::Element => self.handle_element_section(&mut section_reader)?,
                SectionId::Code => self.handle_code_section(&mut section_reader)?,
                SectionId::Data => self.handle_data_section(&mut section_reader)?,
            }
        }

        Ok(())
    }

    fn handle_type_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.func_types = reader.read_usize()?;
        for _ in 0..self.func_types {
            let ft = FuncType::decode(reader, &mut NullVectors::default())?;
            self.val_types += ft.rt1.len();
            self.val_types += ft.rt2.len();
        }
        Ok(())
    }

    fn handle_import_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.imports = reader.read_usize()?;
        for _ in 0..self.imports {
            let import = Import::decode(reader, &mut NullVectors::default())?;
            self.bytes += import.module.len();
            self.bytes += import.name.len();
        }
        Ok(())
    }

    fn handle_function_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.idxs += reader.read_usize()?;
        Ok(())
    }

    fn handle_table_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.table_types = reader.read_usize()?;
        Ok(())
    }

    fn handle_memory_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        let value = reader.read_u32()? as usize;
        if value > 1 {
            return Err(DecodeError::InvalidMemoryCount { value });
        }
        Ok(())
    }

    fn handle_global_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.globals = reader.read_usize()?;
        for _ in 0..self.globals {
            let global = Global::decode(reader, &mut NullVectors::default())?;
            self.instrs += global.init.len();
        }
        Ok(())
    }

    fn handle_export_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.exports = reader.read_usize()?;
        for _ in 0..self.exports {
            let export = Export::decode(reader, &mut NullVectors::default())?;
            self.bytes += export.name.len();
        }
        Ok(())
    }

    fn handle_start_section(&mut self, _reader: &mut Reader) -> Result<(), DecodeError> {
        Ok(())
    }

    fn handle_element_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.elements = reader.read_usize()?;
        for _ in 0..self.elements {
            let elem = Elem::decode(reader, &mut NullVectors::default())?;
            self.instrs += elem.offset.len();
            self.idxs += elem.init.len();
        }
        Ok(())
    }

    fn handle_code_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.codes = reader.read_usize()?;
        for _ in 0..self.codes {
            let code = Code::decode(reader, &mut NullVectors::default())?;
            self.instrs += code.body.len();
            self.locals += code.locals_len;
        }
        Ok(())
    }

    fn handle_data_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.data_segments = reader.read_usize()?;
        for _ in 0..self.data_segments {
            let _data = Data::decode(reader, &mut NullVectors::default())?;
        }
        Ok(())
    }
}
