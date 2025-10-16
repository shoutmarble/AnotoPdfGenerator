### rust_anoto_dots

### rust version of [cheind/py-microdots](https://github.com/cheind/py-microdots)

---


> cargo build   
> > may require  
> >  
> > sudo apt install  
> > pkg-config  
> > libfreetype6-dev  
> > libfontconfig1-dev   
> > librsvg2-bin  

---

```
PS C:\Users\xxxxx\Documents\git\anoto_verify_rust\rs_microdots> .\anoto_dots.exe -h
Generates and verifies Anoto dot patterns

Usage: anoto_dots.exe [OPTIONS]

Options:
  -g, --generate [<height> <width> <sect_u> <sect_v>]
          Generate Anoto dot pattern with shape and section: height width sect_u sect_v (defaults: 9 16 10 2)
  -j, --generate-json <filename>
          Generate from JSON file: filename
  -d, --decode <filename>
          Decode position from 6x6 section file: filename
  -p, --pos <row> <col>
          Extract 6x6 section at position: row col
  -h, --help
          Print help
  -V, --version
          Print version
```

```
PS C:\Users\xxxxx\Documents\git\anoto_verify_rust\rs_microdots> .\anoto_dots.exe -p 10 10 -g 55 55
Matrix size [55, 55]
Requested position (10, 10)
Maximum 6x6 position for this matrix is (49, 49)
```

```
PS C:\Users\xxxxxy\Documents\git\anoto_verify_rust\rs_microdots> .\anoto_dots.exe -d .\output\section_10_10.json
POS (10, 10)
```
