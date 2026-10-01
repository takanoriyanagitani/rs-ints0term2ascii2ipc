use std::io;
use std::sync::Arc;

use io::BufRead;

use io::BufWriter;
use io::Write;

use arrow_ipc::writer::StreamWriter;

use arrow_array::RecordBatch;
use arrow_array::RecordBatchWriter;

use arrow_array::builder::ArrayBuilder;
use arrow_array::builder::StringBuilder;

use arrow_schema::DataType;
use arrow_schema::Field;
use arrow_schema::Schema;
use arrow_schema::SchemaRef;

pub const BAT_SZ_DEFAULT: usize = 8192;
pub const ITEM_SZ: usize = 4;

pub const ITEM_CAP_DEFAULT: usize = BAT_SZ_DEFAULT;
pub const DATA_CAP_DEFAULT: usize = BAT_SZ_DEFAULT * ITEM_SZ;

pub const COLNAME_DEFAULT: &str = "column0";

pub fn int2ascii0term(i: &[u8; 4]) -> Result<&str, io::Error> {
    let sz: usize = i.iter().position(|&b| 0 == b).unwrap_or(i.len());
    std::str::from_utf8(&i[..sz]).map_err(io::Error::other)
}

pub fn rdr2ints<R>(mut rdr: R) -> impl Iterator<Item = Result<[u8; 4], io::Error>>
where
    R: BufRead,
{
    std::iter::from_fn(move || {
        let mut buf: [u8; 4] = [0; 4];
        let rslt: Result<_, _> = rdr.read_exact(&mut buf);
        match rslt {
            Ok(_) => Some(Ok(buf)),
            Err(e) => match e.kind() {
                io::ErrorKind::UnexpectedEof => None,
                _ => Some(Err(e)),
            },
        }
    })
}

pub trait BatSink {
    fn sink(&mut self, bat: &RecordBatch) -> Result<(), io::Error>;
    fn finalize(self) -> Result<(), io::Error>;

    fn ints2ascii0term2array2sink<I>(
        &mut self,
        ints: I,
        sch: SchemaRef,
        item_cap: usize,
        data_cap: usize,
        bat_sz: usize,
    ) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<[u8; 4], io::Error>>,
    {
        let mut bldr: StringBuilder = StringBuilder::with_capacity(item_cap, data_cap);

        for ri in ints {
            let i: [u8; 4] = ri?;
            let ascii: &str = int2ascii0term(&i)?;
            bldr.append_value(ascii);
            if bat_sz <= bldr.len() {
                let bat: RecordBatch =
                    RecordBatch::try_new(sch.clone(), vec![Arc::new(bldr.finish())])
                        .map_err(io::Error::other)?;
                self.sink(&bat)?;
            }
        }

        if 0 < bldr.len() {
            let bat: RecordBatch = RecordBatch::try_new(sch, vec![Arc::new(bldr.finish())])
                .map_err(io::Error::other)?;
            self.sink(&bat)?;
        }

        Ok(())
    }
}

impl<W> BatSink for W
where
    W: RecordBatchWriter,
{
    fn sink(&mut self, bat: &RecordBatch) -> Result<(), io::Error> {
        self.write(bat).map_err(io::Error::other)
    }

    fn finalize(self) -> Result<(), io::Error> {
        self.close().map_err(io::Error::other)
    }
}

pub fn wtr2sink<W>(wtr: W, sch: &SchemaRef) -> Result<impl BatSink, io::Error>
where
    W: Write,
{
    StreamWriter::try_new(wtr, sch).map_err(io::Error::other)
}

pub fn colname2sch(colname: &str) -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new(
        colname,
        DataType::Utf8,
        false,
    )]))
}

pub struct Config {
    pub colname: String,
    pub item_cap: usize,
    pub data_cap: usize,
    pub bat_sz: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            colname: COLNAME_DEFAULT.into(),
            item_cap: ITEM_CAP_DEFAULT,
            data_cap: DATA_CAP_DEFAULT,
            bat_sz: BAT_SZ_DEFAULT,
        }
    }
}

impl Config {
    pub fn rdr2ints2ascii0term2bat2wtr<R, W>(&self, rdr: R, mut wtr: W) -> Result<(), io::Error>
    where
        R: BufRead,
        W: Write,
    {
        let ints = rdr2ints(rdr);
        let sch: SchemaRef = colname2sch(&self.colname);
        let s2: SchemaRef = sch.clone();
        let mut sink = wtr2sink(&mut wtr, &s2)?;
        sink.ints2ascii0term2array2sink(ints, sch, self.item_cap, self.data_cap, self.bat_sz)?;
        sink.finalize()?;
        wtr.flush()
    }
}

impl Config {
    pub fn stdin2ints2ascii0term2bat2stdout(&self) -> Result<(), io::Error> {
        let o = io::stdout();
        let mut ol = o.lock();
        self.rdr2ints2ascii0term2bat2wtr(io::stdin().lock(), BufWriter::new(&mut ol))?;
        ol.flush()
    }
}
