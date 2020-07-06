use crate::gfa::{Link, OptionalField, OptionalFieldValue, Path, Segment, GFA};
use std::fmt::Write;

macro_rules! write_optional {
    ($stream:expr, $path:path, $tag:literal, $val:expr) => {
        if let Some(v) = $val {
            let field = OptionalField {
                tag: $tag.to_string(),
                content: $path(v),
            };
            write!($stream, "{}", field).unwrap_or_else(|err| {
                panic!(
                    "Error writing optional field '{:?}' to stream, {:?}",
                    field, err
                )
            })
        }
    };
}

pub fn write_optional_fields<T: Write>(
    fields: &Vec<OptionalField>,
    stream: &mut T,
) {
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(stream, "\t").unwrap();
        }
        write!(stream, "{}", field).unwrap_or_else(|err| {
            panic!(
                "Error writing optional field '{:?}' to stream, {:?}",
                field, err
            )
        })
    }
}

pub fn write_header<T: Write>(version: &Option<String>, stream: &mut T) {
    if let Some(v) = version {
        write!(stream, "H\tVN:Z:{}", v).unwrap();
    } else {
        write!(stream, "H").unwrap();
    }
}

// Write segment
pub fn write_segment<T: Write>(seg: &Segment, stream: &mut T) {
    use OptionalFieldValue::*;
    write!(stream, "S\t{}\t{}", seg.name, seg.sequence)
        .expect("Error writing segment to stream");

    let seg = seg.clone();
    write_optional!(stream, SignedInt, "LN", seg.segment_length);
    write_optional!(stream, SignedInt, "RC", seg.read_count);
    write_optional!(stream, SignedInt, "FC", seg.fragment_count);
    write_optional!(stream, SignedInt, "KC", seg.kmer_count);
    write_optional!(stream, ByteArray, "SH", seg.sha256);
    write_optional!(stream, PrintableString, "UR", seg.uri);
    write_optional_fields(&seg.optional_fields, stream);
}

pub fn segment_string(seg: &Segment) -> String {
    let mut result = String::new();
    write_segment(seg, &mut result);
    result
}

// Write link
pub fn write_link<T: Write>(link: &Link, stream: &mut T) {
    use OptionalFieldValue::*;

    write!(
        stream,
        "L\t{}\t{}\t{}\t{}\t{}",
        link.from_segment,
        link.from_orient,
        link.to_segment,
        link.to_orient,
        link.overlap
    )
    .expect("Error writing link to stream");

    let link = link.clone();
    write_optional!(stream, SignedInt, "LN", link.map_quality);
    write_optional!(stream, SignedInt, "RC", link.num_mismatches);
    write_optional!(stream, SignedInt, "RC", link.read_count);
    write_optional!(stream, SignedInt, "FC", link.fragment_count);
    write_optional!(stream, SignedInt, "KC", link.kmer_count);
    write_optional!(stream, PrintableString, "SH", link.edge_id);
    write_optional_fields(&link.optional_fields, stream);
}

pub fn link_string(link: &Link) -> String {
    let mut result = String::new();
    write_link(link, &mut result);
    result
}

// Write path
pub fn write_path<T: Write>(path: &Path, stream: &mut T) {
    write!(stream, "P\t{}\t", path.path_name)
        .expect("Error writing path to stream");
    path.segment_names
        .iter()
        .enumerate()
        .for_each(|(i, (n, o))| {
            if i != 0 {
                write!(stream, ",").unwrap();
            }
            write!(stream, "{}{}", n, o).unwrap();
        });
    write!(stream, "\t").unwrap();
    path.overlaps.iter().enumerate().for_each(|(i, o)| {
        if i != 0 {
            write!(stream, ",").unwrap();
        }
        write!(stream, "{}", o).unwrap();
    });

    write_optional_fields(&path.optional_fields, stream);
}

pub fn path_string(path: &Path) -> String {
    let mut result = String::new();
    write_path(path, &mut result);
    result
}

// Write GFA
pub fn write_gfa<T: Write>(gfa: &GFA, stream: &mut T) {
    write_header(&gfa.version, stream);
    writeln!(stream).unwrap();
    gfa.segments.iter().for_each(|s| {
        write_segment(s, stream);
        writeln!(stream).unwrap();
    });

    gfa.paths.iter().for_each(|p| {
        write_path(p, stream);
        writeln!(stream).unwrap();
    });

    gfa.links.iter().for_each(|l| {
        write_link(l, stream);
        writeln!(stream).unwrap();
    });
}

pub fn gfa_string(gfa: &GFA) -> String {
    let mut result = String::new();
    write_gfa(gfa, &mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfa::Orientation;

    #[test]
    fn print_segment() {
        let segment = Segment::new("seg1", "GCCCTA");
        let string = segment_string(&segment);
        assert_eq!(string, "S\tseg1\tGCCCTA");
    }

    #[test]
    fn print_link() {
        let link = Link::new(
            "13",
            Orientation::Forward,
            "552",
            Orientation::Backward,
            "0M",
        );
        let string = link_string(&link);
        assert_eq!(string, "L\t13\t+\t552\t-\t0M");
    }

    #[test]
    fn print_path() {
        let path = Path::new(
            "path1",
            vec!["13+", "51-", "241+"],
            vec!["8M", "1M", "3M"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        let string = path_string(&path);
        assert_eq!(string, "P\tpath1\t13+,51-,241+\t8M,1M,3M");
    }

    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn print_gfa() {
        let in_gfa =
            crate::parser::parse_gfa(&PathBuf::from("./lil.gfa")).unwrap();
        let mut file =
            std::fs::File::open(&PathBuf::from("./lil.gfa")).unwrap();
        let mut file_string = String::new();
        file.read_to_string(&mut file_string).unwrap();

        let string = gfa_string(&in_gfa);

        assert_eq!(string, file_string);
    }
}
