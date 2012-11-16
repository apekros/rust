/*! A codemap is a thing that maps uints to file/line/column positions
 * in a crate. This to make it possible to represent the positions
 * with single-word things, rather than passing records all over the
 * compiler.
 *
 * All represented positions are *absolute* positions within the codemap,
 * not relative positions within a single file.
 */

use dvec::DVec;
use std::serialization::{Serializable,
                         Deserializable,
                         Serializer,
                         Deserializer};

trait Pos {
    static pure fn from_uint(n: uint) -> self;
    pure fn to_uint(&self) -> uint;
}

pub enum BytePos = uint;
pub enum CharPos = uint;

impl BytePos: Pos {
    static pure fn from_uint(n: uint) -> BytePos { BytePos(n) }
    pure fn to_uint(&self) -> uint { **self }
}

impl BytePos: cmp::Eq {
    pure fn eq(other: &BytePos) -> bool {
        *self == **other
    }
    pure fn ne(other: &BytePos) -> bool { !self.eq(other) }
}

impl BytePos: cmp::Ord {
    pure fn lt(other: &BytePos) -> bool { *self < **other }
    pure fn le(other: &BytePos) -> bool { *self <= **other }
    pure fn ge(other: &BytePos) -> bool { *self >= **other }
    pure fn gt(other: &BytePos) -> bool { *self > **other }
}

impl BytePos: Num {
    pure fn add(other: &BytePos) -> BytePos {
        BytePos(*self + **other)
    }
    pure fn sub(other: &BytePos) -> BytePos {
        BytePos(*self - **other)
    }
    pure fn mul(other: &BytePos) -> BytePos {
        BytePos(*self * (**other))
    }
    pure fn div(other: &BytePos) -> BytePos {
        BytePos(*self / **other)
    }
    pure fn modulo(other: &BytePos) -> BytePos {
        BytePos(*self % **other)
    }
    pure fn neg() -> BytePos {
        BytePos(-*self)
    }
    pure fn to_int() -> int { *self as int }
    static pure fn from_int(+n: int) -> BytePos { BytePos(n as uint) }
}

impl BytePos: to_bytes::IterBytes {
    pure fn iter_bytes(+lsb0: bool, f: to_bytes::Cb) {
        (*self).iter_bytes(lsb0, f)
    }
}

impl CharPos: Pos {
    static pure fn from_uint(n: uint) -> CharPos { CharPos(n) }
    pure fn to_uint(&self) -> uint { **self }
}

impl CharPos: cmp::Eq {
    pure fn eq(other: &CharPos) -> bool {
        *self == **other
    }
    pure fn ne(other: &CharPos) -> bool { !self.eq(other) }
}

impl CharPos: cmp::Ord {
    pure fn lt(other: &CharPos) -> bool { *self < **other }
    pure fn le(other: &CharPos) -> bool { *self <= **other }
    pure fn ge(other: &CharPos) -> bool { *self >= **other }
    pure fn gt(other: &CharPos) -> bool { *self > **other }
}

impl CharPos: Num {
    pure fn add(other: &CharPos) -> CharPos {
        CharPos(*self + **other)
    }
    pure fn sub(other: &CharPos) -> CharPos {
        CharPos(*self - **other)
    }
    pure fn mul(other: &CharPos) -> CharPos {
        CharPos(*self * (**other))
    }
    pure fn div(other: &CharPos) -> CharPos {
        CharPos(*self / **other)
    }
    pure fn modulo(other: &CharPos) -> CharPos {
        CharPos(*self % **other)
    }
    pure fn neg() -> CharPos {
        CharPos(-*self)
    }
    pure fn to_int() -> int { *self as int }
    static pure fn from_int(+n: int) -> CharPos { CharPos(n as uint) }
}

impl CharPos: to_bytes::IterBytes {
    pure fn iter_bytes(+lsb0: bool, f: to_bytes::Cb) {
        (*self).iter_bytes(lsb0, f)
    }
}

pub struct span {
    lo: BytePos,
    hi: BytePos,
    expn_info: Option<@ExpnInfo>
}

impl span : cmp::Eq {
    pure fn eq(other: &span) -> bool {
        return self.lo == (*other).lo && self.hi == (*other).hi;
    }
    pure fn ne(other: &span) -> bool { !self.eq(other) }
}

impl<S: Serializer> span: Serializable<S> {
    /* Note #1972 -- spans are serialized but not deserialized */
    fn serialize(&self, _s: &S) { }
}

impl<D: Deserializer> span: Deserializable<D> {
    static fn deserialize(_d: &D) -> span {
        ast_util::dummy_sp()
    }
}

// XXX col shouldn't be CharPos because col is not an absolute location in the
// codemap, and BytePos and CharPos always represent absolute positions
pub struct Loc {
    file: @FileMap, line: uint, col: CharPos
}

pub enum ExpnInfo {
    ExpandedFrom({call_site: span,
                  callie: {name: ~str, span: Option<span>}})
}

pub type FileName = ~str;

pub struct FileLines {
    file: @FileMap,
    lines: ~[uint]
}

pub enum FileSubstr {
    pub FssNone,
    pub FssInternal(span),
    pub FssExternal({filename: ~str, line: uint, col: CharPos})
}

/// Identifies an offset of a multi-byte character in a FileMap
pub struct MultiByteChar {
    /// The absolute offset of the character in the CodeMap
    pos: BytePos,
    /// The number of bytes, >=2
    bytes: uint,
    /// The complete number of 'extra' bytes through this character in the
    /// FileMap
    sum: uint
}

pub struct FileMap {
    name: FileName,
    substr: FileSubstr,
    src: @~str,
    start_pos: BytePos,
    mut lines: ~[BytePos],
    multibyte_chars: DVec<MultiByteChar>
}

pub impl FileMap {
    fn next_line(&self, +pos: BytePos) {
        self.lines.push(pos);
    }

    pub fn get_line(&self, line: int) -> ~str unsafe {
        let begin: BytePos = self.lines[line] - self.start_pos;
        let begin = begin.to_uint();
        let end = match str::find_char_from(*self.src, '\n', begin) {
            Some(e) => e,
            None => str::len(*self.src)
        };
        str::slice(*self.src, begin, end)
    }

    pub fn record_multibyte_char(&self, pos: BytePos, bytes: uint) {
        assert bytes >=2 && bytes <= 4;
        let sum = if self.multibyte_chars.len() > 0 {
            self.multibyte_chars.last().sum
        } else {
            0
        };
        let sum = sum + bytes;
        let mbc = MultiByteChar {
            pos: pos,
            bytes: bytes,
            sum: sum
        };
        self.multibyte_chars.push(mbc);
    }
}

pub struct CodeMap {
    files: DVec<@FileMap>
}

pub impl CodeMap {
    static pub fn new() -> CodeMap {
        CodeMap {
            files: DVec()
        }
    }

    fn new_filemap_w_substr(+filename: FileName, +substr: FileSubstr,
                            src: @~str) -> @FileMap {
        let start_pos = if self.files.len() == 0 {
            0
        } else {
            let last_start = self.files.last().start_pos.to_uint();
            let last_len = self.files.last().src.len();
            last_start + last_len
        };

        let filemap = @FileMap {
            name: filename, substr: substr, src: src,
            start_pos: BytePos(start_pos),
            mut lines: ~[],
            multibyte_chars: DVec()
        };

        self.files.push(filemap);

        return filemap;
    }

    fn new_filemap(+filename: FileName, src: @~str) -> @FileMap {
        return self.new_filemap_w_substr(filename, FssNone, src);
    }

    pub fn mk_substr_filename(&self, sp: span) -> ~str {
        let pos = self.lookup_char_pos(sp.lo);
        return fmt!("<%s:%u:%u>", pos.file.name,
                    pos.line, pos.col.to_uint());
    }

    pub fn lookup_char_pos(&self, +pos: BytePos) -> Loc {
        return self.lookup_pos(pos);
    }

    pub fn lookup_char_pos_adj(&self, +pos: BytePos)
        -> {filename: ~str, line: uint, col: CharPos, file: Option<@FileMap>}
    {
        let loc = self.lookup_char_pos(pos);
        match (loc.file.substr) {
            FssNone => {
                {filename: /* FIXME (#2543) */ copy loc.file.name,
                 line: loc.line,
                 col: loc.col,
                 file: Some(loc.file)}
            }
            FssInternal(sp) => {
                self.lookup_char_pos_adj(
                    sp.lo + (pos - loc.file.start_pos))
            }
            FssExternal(eloc) => {
                {filename: /* FIXME (#2543) */ copy eloc.filename,
                 line: eloc.line + loc.line - 1u,
                 col: if loc.line == 1u {eloc.col + loc.col} else {loc.col},
                 file: None}
            }
        }
    }

    pub fn adjust_span(&self, sp: span) -> span {
        let line = self.lookup_line(sp.lo);
        match (line.fm.substr) {
            FssNone => sp,
            FssInternal(s) => {
                self.adjust_span(span {
                    lo: s.lo + (sp.lo - line.fm.start_pos),
                    hi: s.lo + (sp.hi - line.fm.start_pos),
                    expn_info: sp.expn_info
                })
            }
            FssExternal(_) => sp
        }
    }

    pub fn span_to_str(&self, sp: span) -> ~str {
        let lo = self.lookup_char_pos_adj(sp.lo);
        let hi = self.lookup_char_pos_adj(sp.hi);
        return fmt!("%s:%u:%u: %u:%u", lo.filename,
                    lo.line, lo.col.to_uint(), hi.line, hi.col.to_uint())
    }

    pub fn span_to_filename(&self, sp: span) -> FileName {
        let lo = self.lookup_char_pos(sp.lo);
        return /* FIXME (#2543) */ copy lo.file.name;
    }

    pub fn span_to_lines(&self, sp: span) -> @FileLines {
        let lo = self.lookup_char_pos(sp.lo);
        let hi = self.lookup_char_pos(sp.hi);
        let mut lines = ~[];
        for uint::range(lo.line - 1u, hi.line as uint) |i| {
            lines.push(i);
        };
        return @FileLines {file: lo.file, lines: lines};
    }

    pub fn span_to_snippet(&self, sp: span) -> ~str {
        let begin = self.lookup_byte_offset(sp.lo);
        let end = self.lookup_byte_offset(sp.hi);
        assert begin.fm.start_pos == end.fm.start_pos;
        return str::slice(*begin.fm.src,
                          begin.pos.to_uint(), end.pos.to_uint());
    }

    pub fn get_filemap(&self, filename: ~str) -> @FileMap {
        for self.files.each |fm| { if fm.name == filename { return *fm; } }
        //XXjdm the following triggers a mismatched type bug
        //      (or expected function, found _|_)
        fail; // ("asking for " + filename + " which we don't know about");
    }

}

priv impl CodeMap {

    fn lookup_filemap_idx(&self, +pos: BytePos) -> uint {
        let len = self.files.len();
        let mut a = 0u;
        let mut b = len;
        while b - a > 1u {
            let m = (a + b) / 2u;
            if self.files[m].start_pos > pos {
                b = m;
            } else {
                a = m;
            }
        }
        if (a >= len) {
            fail fmt!("position %u does not resolve to a source location",
                      pos.to_uint())
        }

        return a;
    }

    fn lookup_line(&self, +pos: BytePos)
        -> {fm: @FileMap, line: uint}
    {
        let idx = self.lookup_filemap_idx(pos);
        let f = self.files[idx];
        let mut a = 0u;
        let mut b = vec::len(f.lines);
        while b - a > 1u {
            let m = (a + b) / 2u;
            if f.lines[m] > pos { b = m; } else { a = m; }
        }
        return {fm: f, line: a};
    }

    fn lookup_pos(&self, +pos: BytePos) -> Loc {
        let {fm: f, line: a} = self.lookup_line(pos);
        let line = a + 1u; // Line numbers start at 1
        let chpos = self.bytepos_to_local_charpos(pos);
        let linebpos = f.lines[a];
        let linechpos = self.bytepos_to_local_charpos(linebpos);
        debug!("codemap: byte pos %? is on the line at byte pos %?",
               pos, linebpos);
        debug!("codemap: char pos %? is on the line at char pos %?",
               chpos, linechpos);
        debug!("codemap: byte is on line: %?", line);
        assert chpos >= linechpos;
        return Loc {
            file: f,
            line: line,
            col: chpos - linechpos
        };
    }

    fn span_to_str_no_adj(&self, sp: span) -> ~str {
        let lo = self.lookup_char_pos(sp.lo);
        let hi = self.lookup_char_pos(sp.hi);
        return fmt!("%s:%u:%u: %u:%u", lo.file.name,
                    lo.line, lo.col.to_uint(), hi.line, hi.col.to_uint())
    }

    fn lookup_byte_offset(&self, +bpos: BytePos)
        -> {fm: @FileMap, pos: BytePos} {
        let idx = self.lookup_filemap_idx(bpos);
        let fm = self.files[idx];
        let offset = bpos - fm.start_pos;
        return {fm: fm, pos: offset};
    }

    // Converts an absolute BytePos to a CharPos relative to the file it is
    // located in
    fn bytepos_to_local_charpos(&self, +bpos: BytePos) -> CharPos {
        debug!("codemap: converting %? to char pos", bpos);
        let idx = self.lookup_filemap_idx(bpos);
        let map = self.files[idx];

        // The number of extra bytes due to multibyte chars in the FileMap
        let mut total_extra_bytes = 0;

        for map.multibyte_chars.each |mbc| {
            debug!("codemap: %?-byte char at %?", mbc.bytes, mbc.pos);
            if mbc.pos < bpos {
                total_extra_bytes += mbc.bytes;
                // We should never see a byte position in the middle of a
                // character
                assert bpos == mbc.pos
                    || bpos.to_uint() >= mbc.pos.to_uint() + mbc.bytes;
            } else {
                break;
            }
        }

        CharPos(bpos.to_uint() - total_extra_bytes)
    }
}

//
// Local Variables:
// mode: rust
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
//
