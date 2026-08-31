"""Create a byte-stable ZIP from a directory using only the Python stdlib."""
import pathlib
import sys
import zipfile

source = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2])
files = sorted(path for path in source.rglob("*") if path.is_file())
with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_STORED) as archive:
    for path in files:
        name = path.relative_to(source).as_posix()
        info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        info.create_system = 0
        info.external_attr = 0
        archive.writestr(info, path.read_bytes())
