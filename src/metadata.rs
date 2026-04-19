use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};

pub struct MetadataManager {
    node_file: File,
    edge_file: File,
}

impl MetadataManager {
    pub fn new(base_path: &str) -> std::io::Result<Self> {
        let node_path = format!("{}.nodes.meta", base_path);
        let edge_path = format!("{}.edges.meta", base_path);

        let node_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(node_path)?;
        let edge_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(edge_path)?;

        Ok(Self {
            node_file,
            edge_file,
        })
    }

    pub fn append_node_metadata(&mut self, data: &[u8]) -> std::io::Result<u32> {
        let offset = self.node_file.seek(SeekFrom::End(0))? as u32;
        self.node_file.write_all(data)?;
        Ok(offset)
    }

    pub fn append_edge_metadata(&mut self, data: &[u8]) -> std::io::Result<u32> {
        let offset = self.edge_file.seek(SeekFrom::End(0))? as u32;
        self.edge_file.write_all(data)?;
        Ok(offset)
    }
}
