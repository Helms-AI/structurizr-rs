(module
 (type $0 (func (param i32 i32) (result i32)))
 (type $1 (func))
 (type $2 (func (param i32) (result i32)))
 (type $3 (func (param i32 i32 i32) (result i32)))
 (type $4 (func (param i32 i32)))
 (type $5 (func (param i32 i32 i32)))
 (type $6 (func (result i32)))
 (type $7 (func (param i32 i32 i32 i32)))
 (type $8 (func (param i32 i32 i64)))
 (type $9 (func (param i32)))
 (type $10 (func (param i64 i64 i32 i64 i32) (result i32)))
 (type $11 (func (param f64 i32) (result i32)))
 (type $12 (func (param f32) (result i32)))
 (import "env" "abort" (func $~lib/builtins/abort (param i32 i32 i32 i32)))
 (import "env" "log" (func $assembly/index/log (param i32 i32)))
 (import "env" "get_workspace_name_len" (func $assembly/index/get_workspace_name_len (result i32)))
 (global $~lib/rt/tlsf/ROOT (mut i32) (i32.const 0))
 (global $~lib/rt/tcms/fromSpace (mut i32) (i32.const 0))
 (global $~lib/rt/tcms/total (mut i32) (i32.const 0))
 (global $assembly/index/stats (mut i32) (i32.const 0))
 (global $~lib/util/number/_frc_plus (mut i64) (i64.const 0))
 (global $~lib/util/number/_frc_minus (mut i64) (i64.const 0))
 (global $~lib/util/number/_exp (mut i32) (i32.const 0))
 (global $~lib/util/number/_K (mut i32) (i32.const 0))
 (global $~lib/util/number/_frc_pow (mut i64) (i64.const 0))
 (global $~lib/util/number/_exp_pow (mut i32) (i32.const 0))
 (memory $0 1)
 (data $0 (i32.const 1036) "<")
 (data $0.1 (i32.const 1048) "\02\00\00\00(\00\00\00A\00l\00l\00o\00c\00a\00t\00i\00o\00n\00 \00t\00o\00o\00 \00l\00a\00r\00g\00e")
 (data $1 (i32.const 1100) "<")
 (data $1.1 (i32.const 1112) "\02\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00r\00t\00/\00t\00c\00m\00s\00.\00t\00s")
 (data $2 (i32.const 1164) "<")
 (data $2.1 (i32.const 1176) "\02\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00r\00t\00/\00t\00l\00s\00f\00.\00t\00s")
 (data $4 (i32.const 1260) "l")
 (data $4.1 (i32.const 1272) "\02\00\00\00P\00\00\00=\d8\80\de \00S\00t\00a\00r\00t\00i\00n\00g\00 \00A\00s\00s\00e\00m\00b\00l\00y\00S\00c\00r\00i\00p\00t\00 \00p\00l\00u\00g\00i\00n\00 \00(\00m\00a\00i\00n\00)")
 (data $5 (i32.const 1372) "<")
 (data $5.1 (i32.const 1384) "\02\00\00\00$\00\00\00U\00n\00p\00a\00i\00r\00e\00d\00 \00s\00u\00r\00r\00o\00g\00a\00t\00e")
 (data $6 (i32.const 1436) ",")
 (data $6.1 (i32.const 1448) "\02\00\00\00\1c\00\00\00~\00l\00i\00b\00/\00s\00t\00r\00i\00n\00g\00.\00t\00s")
 (data $7 (i32.const 1484) "<")
 (data $7.1 (i32.const 1496) "\02\00\00\00$\00\00\00I\00n\00d\00e\00x\00 \00o\00u\00t\00 \00o\00f\00 \00r\00a\00n\00g\00e")
 (data $8 (i32.const 1548) "<")
 (data $8.1 (i32.const 1560) "\02\00\00\00$\00\00\00~\00l\00i\00b\00/\00t\00y\00p\00e\00d\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $9 (i32.const 1612) ",")
 (data $9.1 (i32.const 1624) "\02\00\00\00\1c\00\00\00I\00n\00v\00a\00l\00i\00d\00 \00l\00e\00n\00g\00t\00h")
 (data $10 (i32.const 1660) "l")
 (data $10.1 (i32.const 1672) "\02\00\00\00P\00\00\00\0c%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\10%")
 (data $11 (i32.const 1772) "l")
 (data $11.1 (i32.const 1784) "\02\00\00\00P\00\00\00\02% \00 \00=\d8\d8\dc \00A\00s\00s\00e\00m\00b\00l\00y\00S\00c\00r\00i\00p\00t\00 \00T\00y\00p\00e\00-\00S\00a\00f\00e\00 \00G\00r\00e\00e\00t\00e\00r\00 \00\02%")
 (data $12 (i32.const 1884) "l")
 (data $12.1 (i32.const 1896) "\02\00\00\00P\00\00\00\02% \00 \00T\00y\00p\00e\00S\00c\00r\00i\00p\00t\00 \00\92! \00W\00A\00S\00M\00 \00D\00e\00m\00o\00n\00s\00t\00r\00a\00t\00i\00o\00n\00 \00 \00 \00 \00 \00\02%")
 (data $13 (i32.const 1996) "l")
 (data $13.1 (i32.const 2008) "\02\00\00\00P\00\00\00\14%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\18%")
 (data $14 (i32.const 2108) "\1c")
 (data $14.1 (i32.const 2120) "\02")
 (data $15 (i32.const 2140) "\\")
 (data $15.1 (i32.const 2152) "\02\00\00\00@\00\00\00G\00e\00t\00t\00i\00n\00g\00 \00w\00o\00r\00k\00s\00p\00a\00c\00e\00 \00n\00a\00m\00e\00 \00l\00e\00n\00g\00t\00h\00.\00.\00.")
 (data $16 (i32.const 2236) ",")
 (data $16.1 (i32.const 2248) "\02\00\00\00\18\00\00\00G\00o\00t\00 \00l\00e\00n\00g\00t\00h\00:\00 ")
 (data $17 (i32.const 2284) "|")
 (data $17.1 (i32.const 2296) "\02\00\00\00d\00\00\00t\00o\00S\00t\00r\00i\00n\00g\00(\00)\00 \00r\00a\00d\00i\00x\00 \00a\00r\00g\00u\00m\00e\00n\00t\00 \00m\00u\00s\00t\00 \00b\00e\00 \00b\00e\00t\00w\00e\00e\00n\00 \002\00 \00a\00n\00d\00 \003\006")
 (data $18 (i32.const 2412) "<")
 (data $18.1 (i32.const 2424) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00u\00t\00i\00l\00/\00n\00u\00m\00b\00e\00r\00.\00t\00s")
 (data $19 (i32.const 2476) "\1c")
 (data $19.1 (i32.const 2488) "\02\00\00\00\02\00\00\000")
 (data $20 (i32.const 2508) "0\000\000\001\000\002\000\003\000\004\000\005\000\006\000\007\000\008\000\009\001\000\001\001\001\002\001\003\001\004\001\005\001\006\001\007\001\008\001\009\002\000\002\001\002\002\002\003\002\004\002\005\002\006\002\007\002\008\002\009\003\000\003\001\003\002\003\003\003\004\003\005\003\006\003\007\003\008\003\009\004\000\004\001\004\002\004\003\004\004\004\005\004\006\004\007\004\008\004\009\005\000\005\001\005\002\005\003\005\004\005\005\005\006\005\007\005\008\005\009\006\000\006\001\006\002\006\003\006\004\006\005\006\006\006\007\006\008\006\009\007\000\007\001\007\002\007\003\007\004\007\005\007\006\007\007\007\008\007\009\008\000\008\001\008\002\008\003\008\004\008\005\008\006\008\007\008\008\008\009\009\000\009\001\009\002\009\003\009\004\009\005\009\006\009\007\009\008\009\009")
 (data $21 (i32.const 2908) "\1c\04")
 (data $21.1 (i32.const 2920) "\02\00\00\00\00\04\00\000\000\000\001\000\002\000\003\000\004\000\005\000\006\000\007\000\008\000\009\000\00a\000\00b\000\00c\000\00d\000\00e\000\00f\001\000\001\001\001\002\001\003\001\004\001\005\001\006\001\007\001\008\001\009\001\00a\001\00b\001\00c\001\00d\001\00e\001\00f\002\000\002\001\002\002\002\003\002\004\002\005\002\006\002\007\002\008\002\009\002\00a\002\00b\002\00c\002\00d\002\00e\002\00f\003\000\003\001\003\002\003\003\003\004\003\005\003\006\003\007\003\008\003\009\003\00a\003\00b\003\00c\003\00d\003\00e\003\00f\004\000\004\001\004\002\004\003\004\004\004\005\004\006\004\007\004\008\004\009\004\00a\004\00b\004\00c\004\00d\004\00e\004\00f\005\000\005\001\005\002\005\003\005\004\005\005\005\006\005\007\005\008\005\009\005\00a\005\00b\005\00c\005\00d\005\00e\005\00f\006\000\006\001\006\002\006\003\006\004\006\005\006\006\006\007\006\008\006\009\006\00a\006\00b\006\00c\006\00d\006\00e\006\00f\007\000\007\001\007\002\007\003\007\004\007\005\007\006\007\007\007\008\007\009\007\00a\007\00b\007\00c\007\00d\007\00e\007\00f\008\000\008\001\008\002\008\003\008\004\008\005\008\006\008\007\008\008\008\009\008\00a\008\00b\008\00c\008\00d\008\00e\008\00f\009\000\009\001\009\002\009\003\009\004\009\005\009\006\009\007\009\008\009\009\009\00a\009\00b\009\00c\009\00d\009\00e\009\00f\00a\000\00a\001\00a\002\00a\003\00a\004\00a\005\00a\006\00a\007\00a\008\00a\009\00a\00a\00a\00b\00a\00c\00a\00d\00a\00e\00a\00f\00b\000\00b\001\00b\002\00b\003\00b\004\00b\005\00b\006\00b\007\00b\008\00b\009\00b\00a\00b\00b\00b\00c\00b\00d\00b\00e\00b\00f\00c\000\00c\001\00c\002\00c\003\00c\004\00c\005\00c\006\00c\007\00c\008\00c\009\00c\00a\00c\00b\00c\00c\00c\00d\00c\00e\00c\00f\00d\000\00d\001\00d\002\00d\003\00d\004\00d\005\00d\006\00d\007\00d\008\00d\009\00d\00a\00d\00b\00d\00c\00d\00d\00d\00e\00d\00f\00e\000\00e\001\00e\002\00e\003\00e\004\00e\005\00e\006\00e\007\00e\008\00e\009\00e\00a\00e\00b\00e\00c\00e\00d\00e\00e\00e\00f\00f\000\00f\001\00f\002\00f\003\00f\004\00f\005\00f\006\00f\007\00f\008\00f\009\00f\00a\00f\00b\00f\00c\00f\00d\00f\00e\00f\00f")
 (data $22 (i32.const 3964) "\\")
 (data $22.1 (i32.const 3976) "\02\00\00\00H\00\00\000\001\002\003\004\005\006\007\008\009\00a\00b\00c\00d\00e\00f\00g\00h\00i\00j\00k\00l\00m\00n\00o\00p\00q\00r\00s\00t\00u\00v\00w\00x\00y\00z")
 (data $23 (i32.const 4060) "L")
 (data $23.1 (i32.const 4072) "\02\00\00\00.\00\00\00\n\00=\d8\r\dd \00W\00o\00r\00k\00s\00p\00a\00c\00e\00 \00A\00n\00a\00l\00y\00s\00i\00s\00:")
 (data $24 (i32.const 4140) "<")
 (data $24.1 (i32.const 4152) "\02\00\00\00,\00\00\00\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%")
 (data $25 (i32.const 4204) "<")
 (data $25.1 (i32.const 4216) "\02\00\00\00\1e\00\00\00\"  \00N\00a\00m\00e\00 \00l\00e\00n\00g\00t\00h\00:\00 ")
 (data $26 (i32.const 4268) ",")
 (data $26.1 (i32.const 4280) "\02\00\00\00\16\00\00\00 \00c\00h\00a\00r\00a\00c\00t\00e\00r\00s")
 (data $27 (i32.const 4316) ",")
 (data $27.1 (i32.const 4328) "\02\00\00\00\18\00\00\00\"  \00C\00a\00t\00e\00g\00o\00r\00y\00:\00 ")
 (data $28 (i32.const 4364) "\1c")
 (data $28.1 (i32.const 4376) "\02\00\00\00\08\00\00\00T\00i\00n\00y")
 (data $29 (i32.const 4396) "\1c")
 (data $29.1 (i32.const 4408) "\02\00\00\00\n\00\00\00S\00m\00a\00l\00l")
 (data $30 (i32.const 4428) "\1c")
 (data $30.1 (i32.const 4440) "\02\00\00\00\0c\00\00\00M\00e\00d\00i\00u\00m")
 (data $31 (i32.const 4460) "\1c")
 (data $31.1 (i32.const 4472) "\02\00\00\00\n\00\00\00L\00a\00r\00g\00e")
 (data $32 (i32.const 4492) "\1c")
 (data $32.1 (i32.const 4504) "\02\00\00\00\08\00\00\00H\00u\00g\00e")
 (data $33 (i32.const 4524) "<")
 (data $33.1 (i32.const 4536) "\02\00\00\00&\00\00\00\n\00=\d8K\dc \00G\00r\00e\00e\00t\00i\00n\00g\00s\00 \00D\00e\00m\00o\00:")
 (data $34 (i32.const 4588) "<")
 (data $34.1 (i32.const 4600) "\02\00\00\00$\00\00\00\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%")
 (data $35 (i32.const 4652) "\1c")
 (data $35.1 (i32.const 4664) "\02\00\00\00\n\00\00\00H\00e\00l\00l\00o")
 (data $36 (i32.const 4684) "\1c")
 (data $36.1 (i32.const 4696) "\02\00\00\00\02\00\00\00!")
 (data $37 (i32.const 4716) "\1c")
 (data $37.1 (i32.const 4728) "\02\00\00\00\04\00\00\00H\00i")
 (data $38 (i32.const 4748) "\1c")
 (data $38.1 (i32.const 4760) "\02\00\00\00\04\00\00\00!\00!")
 (data $39 (i32.const 4780) ",")
 (data $39.1 (i32.const 4792) "\02\00\00\00\0e\00\00\00W\00e\00l\00c\00o\00m\00e")
 (data $40 (i32.const 4828) "\1c")
 (data $40.1 (i32.const 4840) "\02\00\00\00\06\00\00\00!\00!\00!")
 (data $41 (i32.const 4860) ",")
 (data $41.1 (i32.const 4872) "\02\00\00\00\1a\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $42 (i32.const 4908) "|")
 (data $42.1 (i32.const 4920) "\02\00\00\00^\00\00\00E\00l\00e\00m\00e\00n\00t\00 \00t\00y\00p\00e\00 \00m\00u\00s\00t\00 \00b\00e\00 \00n\00u\00l\00l\00a\00b\00l\00e\00 \00i\00f\00 \00a\00r\00r\00a\00y\00 \00i\00s\00 \00h\00o\00l\00e\00y")
 (data $43 (i32.const 5036) "<")
 (data $43.1 (i32.const 5048) "\02\00\00\00(\00\00\00 \00f\00r\00o\00m\00 \00A\00s\00s\00e\00m\00b\00l\00y\00S\00c\00r\00i\00p\00t")
 (data $44 (i32.const 5100) "\1c")
 (data $44.1 (i32.const 5112) "\02\00\00\00\04\00\00\00\"  ")
 (data $45 (i32.const 5132) "<")
 (data $45.1 (i32.const 5144) "\02\00\00\00&\00\00\00\n\00=\d8\"\dd \00M\00a\00t\00h\00 \00F\00u\00n\00c\00t\00i\00o\00n\00s\00:")
 (data $46 (i32.const 5196) ",")
 (data $46.1 (i32.const 5208) "\02\00\00\00\18\00\00\00\"  \00F\00a\00c\00t\00o\00r\00i\00a\00l\00(")
 (data $47 (i32.const 5244) "\1c")
 (data $47.1 (i32.const 5256) "\02\00\00\00\08\00\00\00)\00 \00=\00 ")
 (data $48 (i32.const 5276) ",")
 (data $48.1 (i32.const 5288) "\02\00\00\00\18\00\00\00\"  \00F\00i\00b\00o\00n\00a\00c\00c\00i\00(")
 (data $49 (i32.const 5324) "\\")
 (data $49.1 (i32.const 5336) "\02\00\00\00F\00\00\00\n\00=\d8\d8\dc \00T\00y\00p\00e\00S\00c\00r\00i\00p\00t\00 \00\92! \00W\00A\00S\00M\00 \00T\00y\00p\00e\00 \00M\00a\00p\00p\00i\00n\00g\00:")
 (data $50 (i32.const 5420) "\\")
 (data $50.1 (i32.const 5432) "\02\00\00\00H\00\00\00\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%")
 (data $51 (i32.const 5516) ",")
 (data $51.1 (i32.const 5528) "\02\00\00\00\14\00\00\00\"  \00i\008\00 \00m\00a\00x\00:\00 ")
 (data $52 (i32.const 5564) ",")
 (data $52.1 (i32.const 5576) "\02\00\00\00\14\00\00\00\"  \00u\008\00 \00m\00a\00x\00:\00 ")
 (data $53 (i32.const 5612) ",")
 (data $53.1 (i32.const 5624) "\02\00\00\00\16\00\00\00\"  \00i\003\002\00 \00m\00a\00x\00:\00 ")
 (data $54 (i32.const 5660) ",")
 (data $54.1 (i32.const 5672) "\02\00\00\00\16\00\00\00\"  \00u\003\002\00 \00m\00a\00x\00:\00 ")
 (data $55 (i32.const 5708) ",")
 (data $55.1 (i32.const 5720) "\02\00\00\00\14\00\00\00\"  \00f\003\002\00 \00p\00i\00:\00 ")
 (data $56 (i32.const 5756) "\1c")
 (data $56.1 (i32.const 5768) "\02\00\00\00\06\00\00\000\00.\000")
 (data $57 (i32.const 5788) "\1c")
 (data $57.1 (i32.const 5800) "\02\00\00\00\06\00\00\00N\00a\00N")
 (data $58 (i32.const 5820) ",")
 (data $58.1 (i32.const 5832) "\02\00\00\00\12\00\00\00-\00I\00n\00f\00i\00n\00i\00t\00y")
 (data $59 (i32.const 5868) ",")
 (data $59.1 (i32.const 5880) "\02\00\00\00\10\00\00\00I\00n\00f\00i\00n\00i\00t\00y")
 (data $61 (i32.const 5976) "\88\02\1c\08\a0\d5\8f\fav\bf>\a2\7f\e1\ae\bav\acU0 \fb\16\8b\ea5\ce]J\89B\cf-;eU\aa\b0k\9a\dfE\1a=\03\cf\1a\e6\ca\c6\9a\c7\17\fep\abO\dc\bc\be\fc\b1w\ff\0c\d6kA\ef\91V\be<\fc\7f\90\ad\1f\d0\8d\83\9aU1(\\Q\d3\b5\c9\a6\ad\8f\acq\9d\cb\8b\ee#w\"\9c\eamSx@\91I\cc\aeW\ce\b6]y\12<\827V\fbM6\94\10\c2O\98H8o\ea\96\90\c7:\82%\cb\85t\d7\f4\97\bf\97\cd\cf\86\a0\e5\ac*\17\98\n4\ef\8e\b25*\fbg8\b2;?\c6\d2\df\d4\c8\84\ba\cd\d3\1a\'D\dd\c5\96\c9%\bb\ce\9fk\93\84\a5b}$l\ac\db\f6\da_\rXf\ab\a3&\f1\c3\de\93\f8\e2\f3\b8\80\ff\aa\a8\ad\b5\b5\8bJ|l\05_b\87S0\c14`\ff\bc\c9U&\ba\91\8c\85N\96\bd~)p$w\f9\df\8f\b8\e5\b8\9f\bd\df\a6\94}t\88\cf_\a9\f8\cf\9b\a8\8f\93pD\b9k\15\0f\bf\f8\f0\08\8a\b611eU%\b0\cd\ac\7f{\d0\c6\e2?\99\06;+*\c4\10\\\e4\d3\92si\99$$\aa\0e\ca\00\83\f2\b5\87\fd\eb\1a\11\92d\08\e5\bc\cc\88Po\t\cc\bc\8c,e\19\e2X\17\b7\d1\00\00\00\00\00\00@\9c\00\00\00\00\10\a5\d4\e8\00\00b\ac\c5\ebx\ad\84\t\94\f8x9?\81\b3\15\07\c9{\ce\97\c0p\\\ea{\ce2~\8fh\80\e9\ab\a48\d2\d5E\"\9a\17&\'O\9f\'\fb\c4\d41\a2c\ed\a8\ad\c8\8c8e\de\b0\dbe\ab\1a\8e\08\c7\83\9a\1dqB\f9\1d]\c4X\e7\1b\a6,iM\92\ea\8dp\1ad\ee\01\daJw\ef\9a\99\a3m\a2\85k}\b4{x\t\f2w\18\ddy\a1\e4T\b4\c2\c5\9b[\92\86[\86=]\96\c8\c5S5\c8\b3\a0\97\fa\\\b4*\95\e3_\a0\99\bd\9fF\de%\8c9\db4\c2\9b\a5\\\9f\98\a3r\9a\c6\f6\ce\be\e9TS\bf\dc\b7\e2A\"\f2\17\f3\fc\88\a5x\\\d3\9b\ce \cc\dfS!{\f3Z\16\98:0\1f\97\dc\b5\a0\e2\96\b3\e3\\S\d1\d9\a8<D\a7\a4\d9|\9b\fb\10D\a4\a7LLv\bb\1a\9c@\b6\ef\8e\ab\8b,\84W\a6\10\ef\1f\d0)1\91\e9\e5\a4\10\9b\9d\0c\9c\a1\fb\9b\10\e7)\f4;b\d9 (\ac\85\cf\a7z^KD\80-\dd\ac\03@\e4!\bf\8f\ffD^/\9cg\8eA\b8\8c\9c\9d\173\d4\a9\1b\e3\b4\92\db\19\9e\d9w\df\ban\bf\96\ebk\ee\f0\9b;\02\87\af")
 (data $62 (i32.const 6672) "<\fbW\fbr\fb\8c\fb\a7\fb\c1\fb\dc\fb\f6\fb\11\fc,\fcF\fca\fc{\fc\96\fc\b1\fc\cb\fc\e6\fc\00\fd\1b\fd5\fdP\fdk\fd\85\fd\a0\fd\ba\fd\d5\fd\ef\fd\n\fe%\fe?\feZ\fet\fe\8f\fe\a9\fe\c4\fe\df\fe\f9\fe\14\ff.\ffI\ffc\ff~\ff\99\ff\b3\ff\ce\ff\e8\ff\03\00\1e\008\00S\00m\00\88\00\a2\00\bd\00\d8\00\f2\00\r\01\'\01B\01\\\01w\01\92\01\ac\01\c7\01\e1\01\fc\01\16\021\02L\02f\02\81\02\9b\02\b6\02\d0\02\eb\02\06\03 \03;\03U\03p\03\8b\03\a5\03\c0\03\da\03\f5\03\0f\04*\04")
 (data $63 (i32.const 6848) "\01\00\00\00\n\00\00\00d\00\00\00\e8\03\00\00\10\'\00\00\a0\86\01\00@B\0f\00\80\96\98\00\00\e1\f5\05\00\ca\9a;")
 (data $64 (i32.const 6892) ",")
 (data $64.1 (i32.const 6904) "\02\00\00\00\12\00\00\00\"  \00f\006\004\00 \00e\00:\00 ")
 (data $65 (i32.const 6940) ",")
 (data $65.1 (i32.const 6952) "\02\00\00\00\16\00\00\00\"  \00b\00o\00o\00l\00e\00a\00n\00:\00 ")
 (data $66 (i32.const 6988) "\1c")
 (data $66.1 (i32.const 7000) "\02\00\00\00\08\00\00\00t\00r\00u\00e")
 (data $67 (i32.const 7020) "\1c")
 (data $67.1 (i32.const 7032) "\02\00\00\00\n\00\00\00f\00a\00l\00s\00e")
 (data $68 (i32.const 7052) ",")
 (data $68.1 (i32.const 7064) "\01\00\00\00\14\00\00\00\01\00\00\00\02\00\00\00\03\00\00\00\04\00\00\00\05")
 (data $69 (i32.const 7100) "<")
 (data $69.1 (i32.const 7112) "\02\00\00\00 \00\00\00\"  \00A\00r\00r\00a\00y\00 \00l\00e\00n\00g\00t\00h\00:\00 ")
 (data $70 (i32.const 7164) "L")
 (data $70.1 (i32.const 7176) "\02\00\00\002\00\00\00\"  \00C\00a\00s\00t\00 \00f\003\002\00(\001\000\00.\007\00)\00 \00t\00o\00 \00i\003\002\00:\00 ")
 (data $71 (i32.const 7244) "<")
 (data $71.1 (i32.const 7256) "\02\00\00\00\1e\00\00\00\n\00=\d8\ca\dc \00S\00t\00a\00t\00i\00s\00t\00i\00c\00s\00:")
 (data $72 (i32.const 7308) "<")
 (data $72.1 (i32.const 7320) "\02\00\00\00\1e\00\00\00\"  \00T\00o\00t\00a\00l\00 \00c\00a\00l\00l\00s\00:\00 ")
 (data $73 (i32.const 7372) "<")
 (data $73.1 (i32.const 7384) "\02\00\00\00$\00\00\00\"  \00A\00v\00e\00r\00a\00g\00e\00 \00l\00e\00n\00g\00t\00h\00:\00 ")
 (data $74 (i32.const 7436) "L")
 (data $74.1 (i32.const 7448) "\02\00\00\008\00\00\00\n\00<\d8\93\df \00A\00s\00s\00e\00m\00b\00l\00y\00S\00c\00r\00i\00p\00t\00 \00F\00e\00a\00t\00u\00r\00e\00s\00:")
 (data $75 (i32.const 7516) "L")
 (data $75.1 (i32.const 7528) "\02\00\00\008\00\00\00\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%\00%")
 (data $76 (i32.const 7596) "\\")
 (data $76.1 (i32.const 7608) "\02\00\00\00F\00\00\00\"  \00T\00y\00p\00e\00S\00c\00r\00i\00p\00t\00 \00s\00y\00n\00t\00a\00x\00 \00w\00i\00t\00h\00 \00W\00A\00S\00M\00 \00t\00y\00p\00e\00s")
 (data $77 (i32.const 7692) "L")
 (data $77.1 (i32.const 7704) "\02\00\00\00:\00\00\00\"  \00G\00a\00r\00b\00a\00g\00e\00 \00c\00o\00l\00l\00e\00c\00t\00i\00o\00n\00 \00i\00n\00c\00l\00u\00d\00e\00d")
 (data $78 (i32.const 7772) "\\")
 (data $78.1 (i32.const 7784) "\02\00\00\00>\00\00\00\"  \00S\00t\00r\00o\00n\00g\00 \00t\00y\00p\00i\00n\00g\00 \00a\00t\00 \00c\00o\00m\00p\00i\00l\00e\00 \00t\00i\00m\00e")
 (data $79 (i32.const 7868) "L")
 (data $79.1 (i32.const 7880) "\02\00\00\006\00\00\00\"  \00~\001\005\00K\00B\00 \00b\00i\00n\00a\00r\00y\00 \00w\00i\00t\00h\00 \00r\00u\00n\00t\00i\00m\00e")
 (data $80 (i32.const 7948) "L")
 (data $80.1 (i32.const 7960) "\02\00\00\008\00\00\00\"  \00F\00a\00m\00i\00l\00i\00a\00r\00 \00t\00o\00 \00w\00e\00b\00 \00d\00e\00v\00e\00l\00o\00p\00e\00r\00s")
 (data $81 (i32.const 8028) "<")
 (data $81.1 (i32.const 8040) "\02\00\00\00*\00\00\00\n\00\05\' \00A\00n\00a\00l\00y\00s\00i\00s\00 \00c\00o\00m\00p\00l\00e\00t\00e\00!")
 (data $82 (i32.const 8092) "l")
 (data $82.1 (i32.const 8104) "\02\00\00\00N\00\00\00=\d8\80\de \00S\00t\00a\00r\00t\00i\00n\00g\00 \00A\00s\00s\00e\00m\00b\00l\00y\00S\00c\00r\00i\00p\00t\00 \00p\00l\00u\00g\00i\00n\00 \00(\00r\00u\00n\00)")
 (export "main" (func $assembly/index/main))
 (export "run" (func $assembly/index/run))
 (export "getHeapSize" (func $assembly/index/getHeapSize))
 (export "getAllocatedMemory" (func $assembly/index/getAllocatedMemory))
 (export "memory" (memory $0))
 (start $~start)
 (func $~lib/rt/tlsf/removeBlock (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $1
  i32.load
  i32.const -4
  i32.and
  local.tee $3
  i32.const 256
  i32.lt_u
  if (result i32)
   local.get $3
   i32.const 4
   i32.shr_u
  else
   i32.const 31
   i32.const 1073741820
   local.get $3
   local.get $3
   i32.const 1073741820
   i32.ge_u
   select
   local.tee $3
   i32.clz
   i32.sub
   local.tee $4
   i32.const 7
   i32.sub
   local.set $2
   local.get $3
   local.get $4
   i32.const 4
   i32.sub
   i32.shr_u
   i32.const 16
   i32.xor
  end
  local.set $3
  local.get $1
  i32.load offset=8
  local.set $5
  local.get $1
  i32.load offset=4
  local.tee $4
  if
   local.get $4
   local.get $5
   i32.store offset=8
  end
  local.get $5
  if
   local.get $5
   local.get $4
   i32.store offset=4
  end
  local.get $1
  local.get $0
  local.get $2
  i32.const 4
  i32.shl
  local.get $3
  i32.add
  i32.const 2
  i32.shl
  i32.add
  local.tee $1
  i32.load offset=96
  i32.eq
  if
   local.get $1
   local.get $5
   i32.store offset=96
   local.get $5
   i32.eqz
   if
    local.get $0
    local.get $2
    i32.const 2
    i32.shl
    i32.add
    local.tee $1
    i32.load offset=4
    i32.const -2
    local.get $3
    i32.rotl
    i32.and
    local.set $3
    local.get $1
    local.get $3
    i32.store offset=4
    local.get $3
    i32.eqz
    if
     local.get $0
     local.get $0
     i32.load
     i32.const -2
     local.get $2
     i32.rotl
     i32.and
     i32.store
    end
   end
  end
 )
 (func $~lib/rt/tlsf/insertBlock (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  local.get $1
  i32.const 4
  i32.add
  local.tee $6
  local.get $1
  i32.load
  local.tee $3
  i32.const -4
  i32.and
  i32.add
  local.tee $4
  i32.load
  local.tee $2
  i32.const 1
  i32.and
  if
   local.get $0
   local.get $4
   call $~lib/rt/tlsf/removeBlock
   local.get $1
   local.get $3
   i32.const 4
   i32.add
   local.get $2
   i32.const -4
   i32.and
   i32.add
   local.tee $3
   i32.store
   local.get $6
   local.get $1
   i32.load
   i32.const -4
   i32.and
   i32.add
   local.tee $4
   i32.load
   local.set $2
  end
  local.get $3
  i32.const 2
  i32.and
  if
   local.get $1
   i32.const 4
   i32.sub
   i32.load
   local.tee $1
   i32.load
   local.set $6
   local.get $0
   local.get $1
   call $~lib/rt/tlsf/removeBlock
   local.get $1
   local.get $6
   i32.const 4
   i32.add
   local.get $3
   i32.const -4
   i32.and
   i32.add
   local.tee $3
   i32.store
  end
  local.get $4
  local.get $2
  i32.const 2
  i32.or
  i32.store
  local.get $4
  i32.const 4
  i32.sub
  local.get $1
  i32.store
  local.get $0
  local.get $3
  i32.const -4
  i32.and
  local.tee $2
  i32.const 256
  i32.lt_u
  if (result i32)
   local.get $2
   i32.const 4
   i32.shr_u
  else
   i32.const 31
   i32.const 1073741820
   local.get $2
   local.get $2
   i32.const 1073741820
   i32.ge_u
   select
   local.tee $2
   i32.clz
   i32.sub
   local.tee $3
   i32.const 7
   i32.sub
   local.set $5
   local.get $2
   local.get $3
   i32.const 4
   i32.sub
   i32.shr_u
   i32.const 16
   i32.xor
  end
  local.tee $2
  local.get $5
  i32.const 4
  i32.shl
  i32.add
  i32.const 2
  i32.shl
  i32.add
  i32.load offset=96
  local.set $3
  local.get $1
  i32.const 0
  i32.store offset=4
  local.get $1
  local.get $3
  i32.store offset=8
  local.get $3
  if
   local.get $3
   local.get $1
   i32.store offset=4
  end
  local.get $0
  local.get $5
  i32.const 4
  i32.shl
  local.get $2
  i32.add
  i32.const 2
  i32.shl
  i32.add
  local.get $1
  i32.store offset=96
  local.get $0
  local.get $0
  i32.load
  i32.const 1
  local.get $5
  i32.shl
  i32.or
  i32.store
  local.get $0
  local.get $5
  i32.const 2
  i32.shl
  i32.add
  local.tee $0
  local.get $0
  i32.load offset=4
  i32.const 1
  local.get $2
  i32.shl
  i32.or
  i32.store offset=4
 )
 (func $~lib/rt/tlsf/addMemory (param $0 i32) (param $1 i32) (param $2 i64)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $1
  i32.const 19
  i32.add
  i32.const -16
  i32.and
  i32.const 4
  i32.sub
  local.set $1
  local.get $0
  i32.load offset=1568
  local.tee $3
  if
   local.get $3
   local.get $1
   i32.const 16
   i32.sub
   local.tee $5
   i32.eq
   if
    local.get $3
    i32.load
    local.set $4
    local.get $5
    local.set $1
   end
  end
  local.get $2
  i32.wrap_i64
  i32.const -16
  i32.and
  local.get $1
  i32.sub
  local.tee $3
  i32.const 20
  i32.lt_u
  if
   return
  end
  local.get $1
  local.get $4
  i32.const 2
  i32.and
  local.get $3
  i32.const 8
  i32.sub
  local.tee $3
  i32.const 1
  i32.or
  i32.or
  i32.store
  local.get $1
  i32.const 0
  i32.store offset=4
  local.get $1
  i32.const 0
  i32.store offset=8
  local.get $1
  i32.const 4
  i32.add
  local.get $3
  i32.add
  local.tee $3
  i32.const 2
  i32.store
  local.get $0
  local.get $3
  i32.store offset=1568
  local.get $0
  local.get $1
  call $~lib/rt/tlsf/insertBlock
 )
 (func $~lib/rt/tlsf/initialize
  (local $0 i32)
  (local $1 i32)
  memory.size
  local.tee $1
  i32.const 0
  i32.le_s
  if (result i32)
   i32.const 1
   local.get $1
   i32.sub
   memory.grow
   i32.const 0
   i32.lt_s
  else
   i32.const 0
  end
  if
   unreachable
  end
  i32.const 8208
  i32.const 0
  i32.store
  i32.const 9776
  i32.const 0
  i32.store
  loop $for-loop|0
   local.get $0
   i32.const 23
   i32.lt_u
   if
    local.get $0
    i32.const 2
    i32.shl
    i32.const 8208
    i32.add
    i32.const 0
    i32.store offset=4
    i32.const 0
    local.set $1
    loop $for-loop|1
     local.get $1
     i32.const 16
     i32.lt_u
     if
      local.get $0
      i32.const 4
      i32.shl
      local.get $1
      i32.add
      i32.const 2
      i32.shl
      i32.const 8208
      i32.add
      i32.const 0
      i32.store offset=96
      local.get $1
      i32.const 1
      i32.add
      local.set $1
      br $for-loop|1
     end
    end
    local.get $0
    i32.const 1
    i32.add
    local.set $0
    br $for-loop|0
   end
  end
  i32.const 8208
  i32.const 9780
  memory.size
  i64.extend_i32_s
  i64.const 16
  i64.shl
  call $~lib/rt/tlsf/addMemory
  i32.const 8208
  global.set $~lib/rt/tlsf/ROOT
 )
 (func $~lib/rt/tlsf/prepareSize (param $0 i32) (result i32)
  local.get $0
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 1056
   i32.const 1184
   i32.const 461
   i32.const 29
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.const 12
  i32.le_u
  if (result i32)
   i32.const 12
  else
   local.get $0
   i32.const 19
   i32.add
   i32.const -16
   i32.and
   i32.const 4
   i32.sub
  end
 )
 (func $~lib/rt/tlsf/searchBlock (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $1
  i32.const 256
  i32.lt_u
  if
   local.get $1
   i32.const 4
   i32.shr_u
   local.set $1
  else
   local.get $1
   i32.const 536870910
   i32.lt_u
   if
    local.get $1
    i32.const 1
    i32.const 27
    local.get $1
    i32.clz
    i32.sub
    i32.shl
    i32.add
    i32.const 1
    i32.sub
    local.set $1
   end
   local.get $1
   i32.const 31
   local.get $1
   i32.clz
   i32.sub
   local.tee $2
   i32.const 4
   i32.sub
   i32.shr_u
   i32.const 16
   i32.xor
   local.set $1
   local.get $2
   i32.const 7
   i32.sub
   local.set $2
  end
  local.get $0
  local.get $2
  i32.const 2
  i32.shl
  i32.add
  i32.load offset=4
  i32.const -1
  local.get $1
  i32.shl
  i32.and
  local.tee $1
  if (result i32)
   local.get $0
   local.get $1
   i32.ctz
   local.get $2
   i32.const 4
   i32.shl
   i32.add
   i32.const 2
   i32.shl
   i32.add
   i32.load offset=96
  else
   local.get $0
   i32.load
   i32.const -1
   local.get $2
   i32.const 1
   i32.add
   i32.shl
   i32.and
   local.tee $1
   if (result i32)
    local.get $0
    local.get $0
    local.get $1
    i32.ctz
    local.tee $0
    i32.const 2
    i32.shl
    i32.add
    i32.load offset=4
    i32.ctz
    local.get $0
    i32.const 4
    i32.shl
    i32.add
    i32.const 2
    i32.shl
    i32.add
    i32.load offset=96
   else
    i32.const 0
   end
  end
 )
 (func $~lib/rt/tlsf/prepareBlock (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $1
  i32.load
  local.tee $3
  i32.const -4
  i32.and
  local.get $2
  i32.sub
  local.tee $4
  i32.const 16
  i32.ge_u
  if
   local.get $1
   local.get $2
   local.get $3
   i32.const 2
   i32.and
   i32.or
   i32.store
   local.get $1
   i32.const 4
   i32.add
   local.get $2
   i32.add
   local.tee $1
   local.get $4
   i32.const 4
   i32.sub
   i32.const 1
   i32.or
   i32.store
   local.get $0
   local.get $1
   call $~lib/rt/tlsf/insertBlock
  else
   local.get $1
   local.get $3
   i32.const -2
   i32.and
   i32.store
   local.get $1
   i32.const 4
   i32.add
   local.get $1
   i32.load
   i32.const -4
   i32.and
   i32.add
   local.tee $0
   local.get $0
   i32.load
   i32.const -3
   i32.and
   i32.store
  end
 )
 (func $~lib/rt/tlsf/allocateBlock (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  local.get $1
  call $~lib/rt/tlsf/prepareSize
  local.tee $2
  call $~lib/rt/tlsf/searchBlock
  local.tee $1
  i32.eqz
  if
   memory.size
   local.tee $3
   local.get $2
   i32.const 256
   i32.ge_u
   if (result i32)
    local.get $2
    i32.const 536870910
    i32.lt_u
    if (result i32)
     local.get $2
     i32.const 1
     i32.const 27
     local.get $2
     i32.clz
     i32.sub
     i32.shl
     i32.add
     i32.const 1
     i32.sub
    else
     local.get $2
    end
   else
    local.get $2
   end
   i32.const 4
   local.get $0
   i32.load offset=1568
   local.get $3
   i32.const 16
   i32.shl
   i32.const 4
   i32.sub
   i32.ne
   i32.shl
   i32.add
   i32.const 65535
   i32.add
   i32.const -65536
   i32.and
   i32.const 16
   i32.shr_u
   local.tee $1
   local.get $1
   local.get $3
   i32.lt_s
   select
   memory.grow
   i32.const 0
   i32.lt_s
   if
    local.get $1
    memory.grow
    i32.const 0
    i32.lt_s
    if
     unreachable
    end
   end
   local.get $0
   local.get $3
   i32.const 16
   i32.shl
   memory.size
   i64.extend_i32_s
   i64.const 16
   i64.shl
   call $~lib/rt/tlsf/addMemory
   local.get $0
   local.get $2
   call $~lib/rt/tlsf/searchBlock
   local.set $1
  end
  local.get $1
  i32.load
  drop
  local.get $0
  local.get $1
  call $~lib/rt/tlsf/removeBlock
  local.get $0
  local.get $1
  local.get $2
  call $~lib/rt/tlsf/prepareBlock
  local.get $1
 )
 (func $~lib/rt/tcms/__new (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $0
  i32.const 1073741804
  i32.gt_u
  if
   i32.const 1056
   i32.const 1120
   i32.const 125
   i32.const 30
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/rt/tlsf/ROOT
  i32.eqz
  if
   call $~lib/rt/tlsf/initialize
  end
  global.get $~lib/rt/tlsf/ROOT
  local.get $0
  i32.const 16
  i32.add
  call $~lib/rt/tlsf/allocateBlock
  local.tee $2
  local.get $1
  i32.store offset=12
  local.get $2
  local.get $0
  i32.store offset=16
  global.get $~lib/rt/tcms/fromSpace
  local.tee $0
  i32.load offset=8
  local.set $1
  local.get $2
  local.get $0
  i32.store offset=4
  local.get $2
  local.get $1
  i32.store offset=8
  local.get $1
  local.get $2
  local.get $1
  i32.load offset=4
  i32.const 3
  i32.and
  i32.or
  i32.store offset=4
  local.get $0
  local.get $2
  i32.store offset=8
  global.get $~lib/rt/tcms/total
  local.get $2
  i32.load
  i32.const -4
  i32.and
  i32.const 4
  i32.add
  i32.add
  global.set $~lib/rt/tcms/total
  local.get $2
  i32.const 20
  i32.add
 )
 (func $assembly/index/logMessage (param $0 i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $0
  local.tee $1
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.add
  local.set $3
  loop $while-continue|0
   local.get $1
   local.get $3
   i32.lt_u
   if
    local.get $1
    i32.load16_u
    local.tee $4
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $2
     i32.const 1
     i32.add
    else
     local.get $4
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $2
      i32.const 2
      i32.add
     else
      local.get $4
      i32.const 64512
      i32.and
      i32.const 55296
      i32.eq
      local.get $1
      i32.const 2
      i32.add
      local.get $3
      i32.lt_u
      i32.and
      if
       local.get $1
       i32.load16_u offset=2
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $2
        i32.const 4
        i32.add
        local.set $2
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        br $while-continue|0
       end
      end
      local.get $2
      i32.const 3
      i32.add
     end
    end
    local.set $2
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    br $while-continue|0
   end
  end
  local.get $2
  i32.const 1
  call $~lib/rt/tcms/__new
  local.set $2
  local.get $0
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.get $1
  i32.add
  local.set $4
  local.get $2
  local.set $0
  loop $while-continue|00
   local.get $1
   local.get $4
   i32.lt_u
   if
    local.get $1
    i32.load16_u
    local.tee $3
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $0
     local.get $3
     i32.store8
     local.get $0
     i32.const 1
     i32.add
    else
     local.get $3
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $0
      local.get $3
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      local.get $3
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $0
      i32.const 2
      i32.add
     else
      local.get $3
      i32.const 63488
      i32.and
      i32.const 55296
      i32.eq
      if
       local.get $3
       i32.const 56320
       i32.lt_u
       local.get $1
       i32.const 2
       i32.add
       local.get $4
       i32.lt_u
       i32.and
       if
        local.get $1
        i32.load16_u offset=2
        local.tee $5
        i32.const 64512
        i32.and
        i32.const 56320
        i32.eq
        if
         local.get $0
         local.get $3
         i32.const 1023
         i32.and
         i32.const 10
         i32.shl
         i32.const 65536
         i32.add
         local.get $5
         i32.const 1023
         i32.and
         i32.or
         local.tee $3
         i32.const 63
         i32.and
         i32.const 128
         i32.or
         i32.const 24
         i32.shl
         local.get $3
         i32.const 6
         i32.shr_u
         i32.const 63
         i32.and
         i32.const 128
         i32.or
         i32.const 16
         i32.shl
         i32.or
         local.get $3
         i32.const 12
         i32.shr_u
         i32.const 63
         i32.and
         i32.const 128
         i32.or
         i32.const 8
         i32.shl
         i32.or
         local.get $3
         i32.const 18
         i32.shr_u
         i32.const 240
         i32.or
         i32.or
         i32.store
         local.get $0
         i32.const 4
         i32.add
         local.set $0
         local.get $1
         i32.const 4
         i32.add
         local.set $1
         br $while-continue|00
        end
       end
      end
      local.get $0
      local.get $3
      i32.const 12
      i32.shr_u
      i32.const 224
      i32.or
      local.get $3
      i32.const 6
      i32.shr_u
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $0
      local.get $3
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=2
      local.get $0
      i32.const 3
      i32.add
     end
    end
    local.set $0
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    br $while-continue|00
   end
  end
  local.get $2
  i32.const 20
  i32.sub
  i32.load offset=16
  local.set $0
  i32.const 12
  i32.const 5
  call $~lib/rt/tcms/__new
  local.tee $1
  local.get $2
  i32.store
  local.get $1
  local.get $0
  i32.store offset=8
  local.get $1
  local.get $2
  i32.store offset=4
  local.get $1
  i32.load offset=4
  local.get $1
  i32.load offset=8
  call $assembly/index/log
 )
 (func $~lib/util/number/utoa32_dec_lut (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  loop $while-continue|0
   local.get $1
   i32.const 10000
   i32.ge_u
   if
    local.get $1
    i32.const 10000
    i32.rem_u
    local.set $3
    local.get $1
    i32.const 10000
    i32.div_u
    local.set $1
    local.get $0
    local.get $2
    i32.const 4
    i32.sub
    local.tee $2
    i32.const 1
    i32.shl
    i32.add
    local.get $3
    i32.const 100
    i32.div_u
    i32.const 2
    i32.shl
    i32.const 2508
    i32.add
    i64.load32_u
    local.get $3
    i32.const 100
    i32.rem_u
    i32.const 2
    i32.shl
    i32.const 2508
    i32.add
    i64.load32_u
    i64.const 32
    i64.shl
    i64.or
    i64.store
    br $while-continue|0
   end
  end
  local.get $1
  i32.const 100
  i32.ge_u
  if
   local.get $0
   local.get $2
   i32.const 2
   i32.sub
   local.tee $2
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 100
   i32.rem_u
   i32.const 2
   i32.shl
   i32.const 2508
   i32.add
   i32.load
   i32.store
   local.get $1
   i32.const 100
   i32.div_u
   local.set $1
  end
  local.get $1
  i32.const 10
  i32.ge_u
  if
   local.get $0
   local.get $2
   i32.const 2
   i32.sub
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 2
   i32.shl
   i32.const 2508
   i32.add
   i32.load
   i32.store
  else
   local.get $0
   local.get $2
   i32.const 1
   i32.sub
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 48
   i32.add
   i32.store16
  end
 )
 (func $~lib/util/number/itoa32 (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i32.eqz
  if
   i32.const 2496
   return
  end
  i32.const 0
  local.get $0
  i32.sub
  local.get $0
  local.get $0
  i32.const 31
  i32.shr_u
  i32.const 1
  i32.shl
  local.tee $0
  select
  local.tee $3
  i32.const 100000
  i32.lt_u
  if (result i32)
   local.get $3
   i32.const 100
   i32.lt_u
   if (result i32)
    local.get $3
    i32.const 10
    i32.ge_u
    i32.const 1
    i32.add
   else
    local.get $3
    i32.const 10000
    i32.ge_u
    i32.const 3
    i32.add
    local.get $3
    i32.const 1000
    i32.ge_u
    i32.add
   end
  else
   local.get $3
   i32.const 10000000
   i32.lt_u
   if (result i32)
    local.get $3
    i32.const 1000000
    i32.ge_u
    i32.const 6
    i32.add
   else
    local.get $3
    i32.const 1000000000
    i32.ge_u
    i32.const 8
    i32.add
    local.get $3
    i32.const 100000000
    i32.ge_u
    i32.add
   end
  end
  local.tee $2
  i32.const 1
  i32.shl
  local.get $0
  i32.add
  i32.const 2
  call $~lib/rt/tcms/__new
  local.tee $1
  local.get $0
  i32.add
  local.get $3
  local.get $2
  call $~lib/util/number/utoa32_dec_lut
  local.get $0
  if
   local.get $1
   i32.const 45
   i32.store16
  end
  local.get $1
 )
 (func $~lib/string/String.__concat (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  i32.const 2128
  local.set $2
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.tee $3
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.tee $4
  i32.add
  local.tee $5
  if
   local.get $5
   i32.const 2
   call $~lib/rt/tcms/__new
   local.tee $2
   local.get $0
   local.get $3
   memory.copy
   local.get $2
   local.get $3
   i32.add
   local.get $1
   local.get $4
   memory.copy
  end
  local.get $2
 )
 (func $assembly/index/GreetingConfig#constructor (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  i32.const 12
  i32.const 6
  call $~lib/rt/tcms/__new
  local.tee $3
  i32.const 0
  i32.store
  local.get $3
  i32.const 0
  i32.store offset=4
  local.get $3
  i32.const 0
  i32.store offset=8
  local.get $3
  local.get $0
  i32.store
  local.get $3
  local.get $1
  i32.store offset=4
  local.get $3
  local.get $2
  i32.store offset=8
  local.get $3
 )
 (func $~lib/rt/__newArray (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  i32.const 2
  i32.shl
  local.tee $4
  i32.const 1
  call $~lib/rt/tcms/__new
  local.set $3
  local.get $2
  if
   local.get $3
   local.get $2
   local.get $4
   memory.copy
  end
  i32.const 16
  local.get $1
  call $~lib/rt/tcms/__new
  local.tee $1
  local.get $3
  i32.store
  local.get $1
  local.get $3
  i32.store offset=4
  local.get $1
  local.get $4
  i32.store offset=8
  local.get $1
  local.get $0
  i32.store offset=12
  local.get $1
 )
 (func $~lib/rt/tlsf/moveBlock (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  local.get $0
  local.get $2
  call $~lib/rt/tlsf/allocateBlock
  local.tee $2
  i32.const 4
  i32.add
  local.get $1
  i32.const 4
  i32.add
  local.get $1
  i32.load
  i32.const -4
  i32.and
  memory.copy
  local.get $1
  i32.const 8204
  i32.ge_u
  if
   local.get $1
   local.get $1
   i32.load
   i32.const 1
   i32.or
   i32.store
   local.get $0
   local.get $1
   call $~lib/rt/tlsf/insertBlock
  end
  local.get $2
 )
 (func $~lib/rt/tcms/__renew (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  (local $8 i32)
  local.get $0
  i32.const 20
  i32.sub
  local.set $2
  local.get $0
  i32.const 8204
  i32.lt_u
  if
   local.get $1
   local.get $2
   i32.load offset=12
   call $~lib/rt/tcms/__new
   local.tee $3
   local.get $0
   local.get $1
   local.get $2
   i32.load offset=16
   local.tee $0
   local.get $0
   local.get $1
   i32.gt_u
   select
   memory.copy
   local.get $3
   return
  end
  local.get $1
  i32.const 1073741804
  i32.gt_u
  if
   i32.const 1056
   i32.const 1120
   i32.const 143
   i32.const 30
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/rt/tcms/total
  local.get $2
  i32.load
  i32.const -4
  i32.and
  i32.const 4
  i32.add
  i32.sub
  global.set $~lib/rt/tcms/total
  global.get $~lib/rt/tlsf/ROOT
  i32.eqz
  if
   call $~lib/rt/tlsf/initialize
  end
  local.get $1
  i32.const 16
  i32.add
  local.set $6
  local.get $0
  i32.const 16
  i32.sub
  local.tee $2
  i32.const 8204
  i32.lt_u
  if
   local.get $2
   i32.const 4
   i32.sub
   local.set $0
   local.get $2
   i32.const 15
   i32.and
   i32.const 1
   local.get $2
   select
   if (result i32)
    i32.const 1
   else
    local.get $0
    i32.load
    i32.const 1
    i32.and
   end
   drop
   global.get $~lib/rt/tlsf/ROOT
   local.get $0
   local.get $6
   call $~lib/rt/tlsf/moveBlock
   local.set $0
  else
   block $__inlined_func$~lib/rt/tlsf/reallocateBlock$154
    local.get $2
    i32.const 4
    i32.sub
    local.set $0
    local.get $2
    i32.const 15
    i32.and
    i32.const 1
    local.get $2
    select
    if (result i32)
     i32.const 1
    else
     local.get $0
     i32.load
     i32.const 1
     i32.and
    end
    drop
    global.get $~lib/rt/tlsf/ROOT
    local.set $3
    local.get $6
    call $~lib/rt/tlsf/prepareSize
    local.tee $4
    local.get $0
    i32.load
    local.tee $7
    i32.const -4
    i32.and
    local.tee $5
    i32.le_u
    if
     local.get $3
     local.get $0
     local.get $4
     call $~lib/rt/tlsf/prepareBlock
     br $__inlined_func$~lib/rt/tlsf/reallocateBlock$154
    end
    local.get $0
    i32.const 4
    i32.add
    local.get $0
    i32.load
    i32.const -4
    i32.and
    i32.add
    local.tee $2
    i32.load
    local.tee $8
    i32.const 1
    i32.and
    if
     local.get $5
     i32.const 4
     i32.add
     local.get $8
     i32.const -4
     i32.and
     i32.add
     local.tee $5
     local.get $4
     i32.ge_u
     if
      local.get $3
      local.get $2
      call $~lib/rt/tlsf/removeBlock
      local.get $0
      local.get $7
      i32.const 3
      i32.and
      local.get $5
      i32.or
      i32.store
      local.get $3
      local.get $0
      local.get $4
      call $~lib/rt/tlsf/prepareBlock
      br $__inlined_func$~lib/rt/tlsf/reallocateBlock$154
     end
    end
    local.get $3
    local.get $0
    local.get $6
    call $~lib/rt/tlsf/moveBlock
    local.set $0
   end
  end
  local.get $0
  i32.const 20
  i32.add
  local.tee $0
  i32.const 20
  i32.sub
  local.tee $2
  local.get $1
  i32.store offset=16
  local.get $2
  i32.load offset=4
  i32.const -4
  i32.and
  local.get $2
  i32.store offset=8
  local.get $2
  i32.load offset=8
  local.tee $1
  local.get $2
  local.get $1
  i32.load offset=4
  i32.const 3
  i32.and
  i32.or
  i32.store offset=4
  global.get $~lib/rt/tcms/total
  local.get $2
  i32.load
  i32.const -4
  i32.and
  i32.const 4
  i32.add
  i32.add
  global.set $~lib/rt/tcms/total
  local.get $0
 )
 (func $~lib/array/Array<assembly/index/GreetingConfig>#__set (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  local.get $1
  local.get $0
  i32.load offset=12
  i32.ge_u
  if
   local.get $1
   i32.const 0
   i32.lt_s
   if
    i32.const 1504
    i32.const 4880
    i32.const 130
    i32.const 22
    call $~lib/builtins/abort
    unreachable
   end
   local.get $1
   i32.const 1
   i32.add
   local.tee $5
   local.get $0
   i32.load offset=8
   local.tee $3
   i32.const 2
   i32.shr_u
   i32.gt_u
   if
    local.get $5
    i32.const 268435455
    i32.gt_u
    if
     i32.const 1632
     i32.const 4880
     i32.const 19
     i32.const 48
     call $~lib/builtins/abort
     unreachable
    end
    local.get $0
    i32.load
    local.tee $4
    i32.const 1073741820
    local.get $3
    i32.const 1
    i32.shl
    local.tee $6
    local.get $6
    i32.const 1073741820
    i32.ge_u
    select
    local.tee $6
    i32.const 8
    local.get $5
    local.get $5
    i32.const 8
    i32.le_u
    select
    i32.const 2
    i32.shl
    local.tee $5
    local.get $5
    local.get $6
    i32.lt_u
    select
    local.tee $5
    call $~lib/rt/tcms/__renew
    local.tee $6
    local.get $3
    i32.add
    i32.const 0
    local.get $5
    local.get $3
    i32.sub
    memory.fill
    local.get $4
    local.get $6
    i32.ne
    if
     local.get $0
     local.get $6
     i32.store
     local.get $0
     local.get $6
     i32.store offset=4
    end
    local.get $0
    local.get $5
    i32.store offset=8
   end
   local.get $0
   local.get $1
   i32.const 1
   i32.add
   i32.store offset=12
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.const 2
  i32.shl
  i32.add
  local.get $2
  i32.store
 )
 (func $~lib/array/Array<assembly/index/GreetingConfig>#__get (param $0 i32) (param $1 i32) (result i32)
  local.get $1
  local.get $0
  i32.load offset=12
  i32.ge_u
  if
   i32.const 1504
   i32.const 4880
   i32.const 114
   i32.const 42
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.const 2
  i32.shl
  i32.add
  i32.load
  local.tee $0
  i32.eqz
  if
   i32.const 4928
   i32.const 4880
   i32.const 118
   i32.const 40
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
 )
 (func $assembly/index/factorial (param $0 i32) (result i32)
  local.get $0
  i32.const 1
  i32.le_s
  if
   i32.const 1
   return
  end
  local.get $0
  i32.const 1
  i32.sub
  call $assembly/index/factorial
  local.get $0
  i32.mul
 )
 (func $~lib/util/number/utoa32 (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  local.get $0
  i32.eqz
  if
   i32.const 2496
   return
  end
  local.get $0
  i32.const 100000
  i32.lt_u
  if (result i32)
   local.get $0
   i32.const 100
   i32.lt_u
   if (result i32)
    local.get $0
    i32.const 10
    i32.ge_u
    i32.const 1
    i32.add
   else
    local.get $0
    i32.const 10000
    i32.ge_u
    i32.const 3
    i32.add
    local.get $0
    i32.const 1000
    i32.ge_u
    i32.add
   end
  else
   local.get $0
   i32.const 10000000
   i32.lt_u
   if (result i32)
    local.get $0
    i32.const 1000000
    i32.ge_u
    i32.const 6
    i32.add
   else
    local.get $0
    i32.const 1000000000
    i32.ge_u
    i32.const 8
    i32.add
    local.get $0
    i32.const 100000000
    i32.ge_u
    i32.add
   end
  end
  local.tee $2
  i32.const 1
  i32.shl
  i32.const 2
  call $~lib/rt/tcms/__new
  local.tee $1
  local.get $0
  local.get $2
  call $~lib/util/number/utoa32_dec_lut
  local.get $1
 )
 (func $~lib/util/number/genDigits (param $0 i64) (param $1 i64) (param $2 i32) (param $3 i64) (param $4 i32) (result i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i64)
  (local $8 i32)
  (local $9 i64)
  (local $10 i64)
  (local $11 i32)
  (local $12 i64)
  local.get $1
  local.get $0
  i64.sub
  local.set $10
  i64.const 1
  i32.const 0
  local.get $2
  i32.sub
  local.tee $11
  i64.extend_i32_s
  local.tee $0
  i64.shl
  local.tee $7
  i64.const 1
  i64.sub
  local.tee $12
  local.get $1
  i64.and
  local.set $9
  local.get $1
  local.get $0
  i64.shr_u
  i32.wrap_i64
  local.tee $5
  i32.const 100000
  i32.lt_u
  if (result i32)
   local.get $5
   i32.const 100
   i32.lt_u
   if (result i32)
    local.get $5
    i32.const 10
    i32.ge_u
    i32.const 1
    i32.add
   else
    local.get $5
    i32.const 10000
    i32.ge_u
    i32.const 3
    i32.add
    local.get $5
    i32.const 1000
    i32.ge_u
    i32.add
   end
  else
   local.get $5
   i32.const 10000000
   i32.lt_u
   if (result i32)
    local.get $5
    i32.const 1000000
    i32.ge_u
    i32.const 6
    i32.add
   else
    local.get $5
    i32.const 1000000000
    i32.ge_u
    i32.const 8
    i32.add
    local.get $5
    i32.const 100000000
    i32.ge_u
    i32.add
   end
  end
  local.set $8
  loop $while-continue|0
   local.get $8
   i32.const 0
   i32.gt_s
   if
    block $break|1
     block $case10|1
      block $case9|1
       block $case8|1
        block $case7|1
         block $case6|1
          block $case5|1
           block $case4|1
            block $case3|1
             block $case2|1
              block $case1|1
               block $case0|1
                local.get $8
                i32.const 1
                i32.sub
                br_table $case9|1 $case8|1 $case7|1 $case6|1 $case5|1 $case4|1 $case3|1 $case2|1 $case1|1 $case0|1 $case10|1
               end
               local.get $5
               i32.const 1000000000
               i32.div_u
               local.set $6
               local.get $5
               i32.const 1000000000
               i32.rem_u
               local.set $5
               br $break|1
              end
              local.get $5
              i32.const 100000000
              i32.div_u
              local.set $6
              local.get $5
              i32.const 100000000
              i32.rem_u
              local.set $5
              br $break|1
             end
             local.get $5
             i32.const 10000000
             i32.div_u
             local.set $6
             local.get $5
             i32.const 10000000
             i32.rem_u
             local.set $5
             br $break|1
            end
            local.get $5
            i32.const 1000000
            i32.div_u
            local.set $6
            local.get $5
            i32.const 1000000
            i32.rem_u
            local.set $5
            br $break|1
           end
           local.get $5
           i32.const 100000
           i32.div_u
           local.set $6
           local.get $5
           i32.const 100000
           i32.rem_u
           local.set $5
           br $break|1
          end
          local.get $5
          i32.const 10000
          i32.div_u
          local.set $6
          local.get $5
          i32.const 10000
          i32.rem_u
          local.set $5
          br $break|1
         end
         local.get $5
         i32.const 1000
         i32.div_u
         local.set $6
         local.get $5
         i32.const 1000
         i32.rem_u
         local.set $5
         br $break|1
        end
        local.get $5
        i32.const 100
        i32.div_u
        local.set $6
        local.get $5
        i32.const 100
        i32.rem_u
        local.set $5
        br $break|1
       end
       local.get $5
       i32.const 10
       i32.div_u
       local.set $6
       local.get $5
       i32.const 10
       i32.rem_u
       local.set $5
       br $break|1
      end
      local.get $5
      local.set $6
      i32.const 0
      local.set $5
      br $break|1
     end
     i32.const 0
     local.set $6
    end
    local.get $4
    local.get $6
    i32.or
    if
     local.get $4
     local.tee $2
     i32.const 1
     i32.add
     local.set $4
     local.get $2
     i32.const 1
     i32.shl
     i32.const 5920
     i32.add
     local.get $6
     i32.const 65535
     i32.and
     i32.const 48
     i32.add
     i32.store16
    end
    local.get $8
    i32.const 1
    i32.sub
    local.set $8
    local.get $3
    local.get $5
    i64.extend_i32_u
    local.get $11
    i64.extend_i32_s
    local.tee $1
    i64.shl
    local.get $9
    i64.add
    local.tee $0
    i64.ge_u
    if
     global.get $~lib/util/number/_K
     local.get $8
     i32.add
     global.set $~lib/util/number/_K
     local.get $8
     i32.const 2
     i32.shl
     i32.const 6848
     i32.add
     i64.load32_u
     local.get $1
     i64.shl
     local.set $7
     local.get $4
     i32.const 1
     i32.shl
     i32.const 5918
     i32.add
     local.tee $2
     i32.load16_u
     local.set $6
     loop $while-continue|3
      local.get $0
      local.get $10
      i64.lt_u
      local.get $3
      local.get $0
      i64.sub
      local.get $7
      i64.ge_u
      i32.and
      if (result i32)
       local.get $10
       local.get $0
       local.get $7
       i64.add
       local.tee $1
       i64.gt_u
       local.get $10
       local.get $0
       i64.sub
       local.get $1
       local.get $10
       i64.sub
       i64.gt_u
       i32.or
      else
       i32.const 0
      end
      if
       local.get $6
       i32.const 1
       i32.sub
       local.set $6
       local.get $0
       local.get $7
       i64.add
       local.set $0
       br $while-continue|3
      end
     end
     local.get $2
     local.get $6
     i32.store16
     local.get $4
     return
    end
    br $while-continue|0
   end
  end
  loop $while-continue|4
   local.get $3
   i64.const 10
   i64.mul
   local.set $3
   local.get $9
   i64.const 10
   i64.mul
   local.tee $1
   local.get $11
   i64.extend_i32_s
   i64.shr_u
   local.tee $0
   local.get $4
   i64.extend_i32_s
   i64.or
   i64.const 0
   i64.ne
   if
    local.get $4
    local.tee $2
    i32.const 1
    i32.add
    local.set $4
    local.get $2
    i32.const 1
    i32.shl
    i32.const 5920
    i32.add
    local.get $0
    i32.wrap_i64
    i32.const 65535
    i32.and
    i32.const 48
    i32.add
    i32.store16
   end
   local.get $8
   i32.const 1
   i32.sub
   local.set $8
   local.get $1
   local.get $12
   i64.and
   local.tee $9
   local.get $3
   i64.ge_u
   br_if $while-continue|4
  end
  global.get $~lib/util/number/_K
  local.get $8
  i32.add
  global.set $~lib/util/number/_K
  local.get $10
  i32.const 0
  local.get $8
  i32.sub
  i32.const 2
  i32.shl
  i32.const 6848
  i32.add
  i64.load32_u
  i64.mul
  local.set $1
  local.get $4
  i32.const 1
  i32.shl
  i32.const 5918
  i32.add
  local.tee $2
  i32.load16_u
  local.set $6
  loop $while-continue|6
   local.get $1
   local.get $9
   i64.gt_u
   local.get $3
   local.get $9
   i64.sub
   local.get $7
   i64.ge_u
   i32.and
   if (result i32)
    local.get $1
    local.get $7
    local.get $9
    i64.add
    local.tee $0
    i64.gt_u
    local.get $1
    local.get $9
    i64.sub
    local.get $0
    local.get $1
    i64.sub
    i64.gt_u
    i32.or
   else
    i32.const 0
   end
   if
    local.get $6
    i32.const 1
    i32.sub
    local.set $6
    local.get $7
    local.get $9
    i64.add
    local.set $9
    br $while-continue|6
   end
  end
  local.get $2
  local.get $6
  i32.store16
  local.get $4
 )
 (func $~lib/util/number/prettify (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  (local $4 i32)
  local.get $2
  i32.eqz
  if
   local.get $0
   local.get $1
   i32.const 1
   i32.shl
   i32.add
   i32.const 3145774
   i32.store
   local.get $1
   i32.const 2
   i32.add
   return
  end
  local.get $1
  local.get $2
  i32.add
  local.tee $3
  i32.const 21
  i32.le_s
  local.get $1
  local.get $3
  i32.le_s
  i32.and
  if (result i32)
   loop $for-loop|0
    local.get $1
    local.get $3
    i32.lt_s
    if
     local.get $0
     local.get $1
     i32.const 1
     i32.shl
     i32.add
     i32.const 48
     i32.store16
     local.get $1
     i32.const 1
     i32.add
     local.set $1
     br $for-loop|0
    end
   end
   local.get $0
   local.get $3
   i32.const 1
   i32.shl
   i32.add
   i32.const 3145774
   i32.store
   local.get $3
   i32.const 2
   i32.add
  else
   local.get $3
   i32.const 21
   i32.le_s
   local.get $3
   i32.const 0
   i32.gt_s
   i32.and
   if (result i32)
    local.get $0
    local.get $3
    i32.const 1
    i32.shl
    i32.add
    local.tee $0
    i32.const 2
    i32.add
    local.get $0
    i32.const 0
    local.get $2
    i32.sub
    i32.const 1
    i32.shl
    memory.copy
    local.get $0
    i32.const 46
    i32.store16
    local.get $1
    i32.const 1
    i32.add
   else
    local.get $3
    i32.const 0
    i32.le_s
    local.get $3
    i32.const -6
    i32.gt_s
    i32.and
    if (result i32)
     local.get $0
     i32.const 2
     local.get $3
     i32.sub
     local.tee $3
     i32.const 1
     i32.shl
     i32.add
     local.get $0
     local.get $1
     i32.const 1
     i32.shl
     memory.copy
     local.get $0
     i32.const 3014704
     i32.store
     i32.const 2
     local.set $2
     loop $for-loop|1
      local.get $2
      local.get $3
      i32.lt_s
      if
       local.get $0
       local.get $2
       i32.const 1
       i32.shl
       i32.add
       i32.const 48
       i32.store16
       local.get $2
       i32.const 1
       i32.add
       local.set $2
       br $for-loop|1
      end
     end
     local.get $1
     local.get $3
     i32.add
    else
     local.get $1
     i32.const 1
     i32.eq
     if
      local.get $0
      i32.const 101
      i32.store16 offset=2
      local.get $0
      i32.const 4
      i32.add
      local.tee $2
      local.get $3
      i32.const 1
      i32.sub
      local.tee $0
      i32.const 0
      i32.lt_s
      local.tee $3
      if
       i32.const 0
       local.get $0
       i32.sub
       local.set $0
      end
      local.get $0
      local.get $0
      i32.const 100000
      i32.lt_u
      if (result i32)
       local.get $0
       i32.const 100
       i32.lt_u
       if (result i32)
        local.get $0
        i32.const 10
        i32.ge_u
        i32.const 1
        i32.add
       else
        local.get $0
        i32.const 10000
        i32.ge_u
        i32.const 3
        i32.add
        local.get $0
        i32.const 1000
        i32.ge_u
        i32.add
       end
      else
       local.get $0
       i32.const 10000000
       i32.lt_u
       if (result i32)
        local.get $0
        i32.const 1000000
        i32.ge_u
        i32.const 6
        i32.add
       else
        local.get $0
        i32.const 1000000000
        i32.ge_u
        i32.const 8
        i32.add
        local.get $0
        i32.const 100000000
        i32.ge_u
        i32.add
       end
      end
      i32.const 1
      i32.add
      local.tee $1
      call $~lib/util/number/utoa32_dec_lut
      local.get $2
      i32.const 45
      i32.const 43
      local.get $3
      select
      i32.store16
     else
      local.get $0
      i32.const 4
      i32.add
      local.get $0
      i32.const 2
      i32.add
      local.get $1
      i32.const 1
      i32.shl
      local.tee $2
      i32.const 2
      i32.sub
      memory.copy
      local.get $0
      i32.const 46
      i32.store16 offset=2
      local.get $0
      local.get $2
      i32.add
      local.tee $0
      i32.const 101
      i32.store16 offset=2
      local.get $0
      i32.const 4
      i32.add
      local.tee $4
      local.get $3
      i32.const 1
      i32.sub
      local.tee $0
      i32.const 0
      i32.lt_s
      local.tee $2
      if
       i32.const 0
       local.get $0
       i32.sub
       local.set $0
      end
      local.get $0
      local.get $0
      i32.const 100000
      i32.lt_u
      if (result i32)
       local.get $0
       i32.const 100
       i32.lt_u
       if (result i32)
        local.get $0
        i32.const 10
        i32.ge_u
        i32.const 1
        i32.add
       else
        local.get $0
        i32.const 10000
        i32.ge_u
        i32.const 3
        i32.add
        local.get $0
        i32.const 1000
        i32.ge_u
        i32.add
       end
      else
       local.get $0
       i32.const 10000000
       i32.lt_u
       if (result i32)
        local.get $0
        i32.const 1000000
        i32.ge_u
        i32.const 6
        i32.add
       else
        local.get $0
        i32.const 1000000000
        i32.ge_u
        i32.const 8
        i32.add
        local.get $0
        i32.const 100000000
        i32.ge_u
        i32.add
       end
      end
      i32.const 1
      i32.add
      local.tee $0
      call $~lib/util/number/utoa32_dec_lut
      local.get $4
      i32.const 45
      i32.const 43
      local.get $2
      select
      i32.store16
      local.get $0
      local.get $1
      i32.add
      local.set $1
     end
     local.get $1
     i32.const 2
     i32.add
    end
   end
  end
 )
 (func $~lib/util/number/dtoa_core (param $0 f64) (param $1 i32) (result i32)
  (local $2 i64)
  (local $3 i32)
  (local $4 i64)
  (local $5 i32)
  (local $6 i64)
  (local $7 i64)
  (local $8 i64)
  (local $9 i32)
  (local $10 i32)
  (local $11 i64)
  (local $12 i64)
  (local $13 i64)
  (local $14 i64)
  (local $15 i64)
  local.get $0
  f64.const 0
  f64.lt
  local.tee $3
  if
   i32.const 5920
   i32.const 45
   i32.store16
   local.get $0
   f64.neg
   local.set $0
  end
  local.get $1
  if (result i32)
   local.get $0
   f32.demote_f64
   i32.reinterpret_f32
   local.tee $5
   i32.const 2139095040
   i32.and
   i32.const 23
   i32.shr_u
   local.set $9
   local.get $5
   i32.const 8388607
   i32.and
   i64.extend_i32_u
   local.get $9
   i32.const 0
   i32.ne
   i64.extend_i32_u
   i64.const 23
   i64.shl
   i64.add
   local.set $2
   local.get $9
   i32.const 1
   local.get $9
   select
   i32.const 150
   i32.sub
  else
   local.get $0
   i64.reinterpret_f64
   local.tee $2
   i64.const 9218868437227405312
   i64.and
   i64.const 52
   i64.shr_u
   i32.wrap_i64
   local.set $5
   local.get $2
   i64.const 4503599627370495
   i64.and
   local.get $5
   i32.const 0
   i32.ne
   i64.extend_i32_u
   i64.const 52
   i64.shl
   i64.add
   local.set $2
   local.get $5
   i32.const 1
   local.get $5
   select
   i32.const 1075
   i32.sub
  end
  local.tee $9
  i32.const 1
  i32.sub
  local.get $2
  i64.const 1
  i64.shl
  i64.const 1
  i64.add
  local.tee $4
  i64.clz
  i32.wrap_i64
  local.tee $10
  i32.sub
  local.set $5
  local.get $4
  local.get $10
  i64.extend_i32_s
  i64.shl
  global.set $~lib/util/number/_frc_plus
  local.get $2
  local.get $2
  i64.const 8388608
  i64.const 4503599627370496
  local.get $1
  select
  i64.eq
  i32.const 1
  i32.add
  local.tee $1
  i64.extend_i32_s
  i64.shl
  i64.const 1
  i64.sub
  local.get $9
  local.get $1
  i32.sub
  local.get $5
  i32.sub
  i64.extend_i32_s
  i64.shl
  global.set $~lib/util/number/_frc_minus
  local.get $5
  global.set $~lib/util/number/_exp
  i32.const 348
  i32.const -61
  global.get $~lib/util/number/_exp
  i32.sub
  f64.convert_i32_s
  f64.const 0.30102999566398114
  f64.mul
  f64.const 347
  f64.add
  local.tee $0
  i32.trunc_sat_f64_s
  local.tee $1
  local.get $1
  f64.convert_i32_s
  local.get $0
  f64.ne
  i32.add
  i32.const 3
  i32.shr_s
  i32.const 1
  i32.add
  local.tee $1
  i32.const 3
  i32.shl
  local.tee $5
  i32.sub
  global.set $~lib/util/number/_K
  local.get $5
  i32.const 5976
  i32.add
  i64.load
  global.set $~lib/util/number/_frc_pow
  local.get $1
  i32.const 1
  i32.shl
  i32.const 6672
  i32.add
  i32.load16_s
  global.set $~lib/util/number/_exp_pow
  local.get $2
  local.get $2
  i64.clz
  i64.shl
  local.tee $2
  i64.const 4294967295
  i64.and
  local.set $6
  global.get $~lib/util/number/_frc_pow
  local.tee $11
  i64.const 4294967295
  i64.and
  local.tee $12
  local.get $2
  i64.const 32
  i64.shr_u
  local.tee $2
  i64.mul
  local.get $6
  local.get $12
  i64.mul
  i64.const 32
  i64.shr_u
  i64.add
  local.set $7
  global.get $~lib/util/number/_frc_plus
  local.tee $4
  i64.const 4294967295
  i64.and
  local.set $13
  local.get $4
  i64.const 32
  i64.shr_u
  local.tee $4
  local.get $12
  i64.mul
  local.get $12
  local.get $13
  i64.mul
  i64.const 32
  i64.shr_u
  i64.add
  local.set $8
  global.get $~lib/util/number/_frc_minus
  local.tee $14
  i64.const 4294967295
  i64.and
  local.set $15
  local.get $14
  i64.const 32
  i64.shr_u
  local.tee $14
  local.get $12
  i64.mul
  local.get $12
  local.get $15
  i64.mul
  i64.const 32
  i64.shr_u
  i64.add
  local.set $12
  local.get $3
  i32.const 1
  i32.shl
  i32.const 5920
  i32.add
  local.get $2
  local.get $11
  i64.const 32
  i64.shr_u
  local.tee $2
  i64.mul
  local.get $7
  i64.const 32
  i64.shr_u
  i64.add
  local.get $2
  local.get $6
  i64.mul
  local.get $7
  i64.const 4294967295
  i64.and
  i64.add
  i64.const 2147483647
  i64.add
  i64.const 32
  i64.shr_u
  i64.add
  local.get $2
  local.get $4
  i64.mul
  local.get $8
  i64.const 32
  i64.shr_u
  i64.add
  local.get $2
  local.get $13
  i64.mul
  local.get $8
  i64.const 4294967295
  i64.and
  i64.add
  i64.const 2147483647
  i64.add
  i64.const 32
  i64.shr_u
  i64.add
  i64.const 1
  i64.sub
  local.tee $4
  global.get $~lib/util/number/_exp_pow
  global.get $~lib/util/number/_exp
  i32.add
  i32.const -64
  i32.sub
  local.get $4
  local.get $2
  local.get $14
  i64.mul
  local.get $12
  i64.const 32
  i64.shr_u
  i64.add
  local.get $2
  local.get $15
  i64.mul
  local.get $12
  i64.const 4294967295
  i64.and
  i64.add
  i64.const 2147483647
  i64.add
  i64.const 32
  i64.shr_u
  i64.add
  i64.const 1
  i64.add
  i64.sub
  local.get $3
  call $~lib/util/number/genDigits
  local.get $3
  i32.sub
  global.get $~lib/util/number/_K
  call $~lib/util/number/prettify
  local.get $3
  i32.add
 )
 (func $~lib/number/F32#toString (param $0 f32) (result i32)
  (local $1 f64)
  (local $2 i32)
  (local $3 i32)
  i32.const 5776
  local.set $2
  block $~lib/util/number/dtoa_impl|inlined.0
   local.get $0
   f64.promote_f32
   local.tee $1
   f64.const 0
   f64.eq
   br_if $~lib/util/number/dtoa_impl|inlined.0
   local.get $1
   local.get $1
   f64.sub
   f64.const 0
   f64.ne
   if
    i32.const 5808
    local.set $2
    local.get $1
    local.get $1
    f64.ne
    br_if $~lib/util/number/dtoa_impl|inlined.0
    i32.const 5840
    i32.const 5888
    local.get $1
    f64.const 0
    f64.lt
    select
    local.set $2
    br $~lib/util/number/dtoa_impl|inlined.0
   end
   local.get $1
   i32.const 1
   call $~lib/util/number/dtoa_core
   i32.const 1
   i32.shl
   local.tee $3
   i32.const 2
   call $~lib/rt/tcms/__new
   local.tee $2
   i32.const 5920
   local.get $3
   memory.copy
  end
  local.get $2
 )
 (func $assembly/index/analyze
  (local $0 i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  (local $8 i32)
  i32.const 1680
  call $assembly/index/logMessage
  i32.const 1792
  call $assembly/index/logMessage
  i32.const 1904
  call $assembly/index/logMessage
  i32.const 2016
  call $assembly/index/logMessage
  i32.const 2128
  call $assembly/index/logMessage
  i32.const 2160
  call $assembly/index/logMessage
  i32.const 2256
  call $assembly/index/get_workspace_name_len
  local.tee $8
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  global.get $assembly/index/stats
  local.tee $0
  local.get $0
  i32.load
  i32.const 1
  i32.add
  i32.store
  local.get $0
  local.get $0
  i32.load offset=4
  local.get $8
  i32.add
  i32.store offset=4
  local.get $0
  local.get $0
  i32.load offset=4
  f32.convert_i32_s
  local.get $0
  i32.load
  f32.convert_i32_s
  f32.div
  f32.store offset=8
  i32.const 4080
  call $assembly/index/logMessage
  i32.const 4160
  call $assembly/index/logMessage
  i32.const 4224
  local.get $8
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  i32.const 4288
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 4336
  block $__inlined_func$assembly/index/categorizeWorkspace$127 (result i32)
   i32.const 4384
   local.get $8
   i32.const 5
   i32.le_s
   br_if $__inlined_func$assembly/index/categorizeWorkspace$127
   drop
   i32.const 4416
   local.get $8
   i32.const 10
   i32.le_s
   br_if $__inlined_func$assembly/index/categorizeWorkspace$127
   drop
   i32.const 4448
   local.get $8
   i32.const 20
   i32.le_s
   br_if $__inlined_func$assembly/index/categorizeWorkspace$127
   drop
   i32.const 4480
   local.get $8
   i32.const 30
   i32.le_s
   br_if $__inlined_func$assembly/index/categorizeWorkspace$127
   drop
   i32.const 4512
  end
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 4544
  call $assembly/index/logMessage
  i32.const 4608
  call $assembly/index/logMessage
  i32.const 3
  i32.const 7
  i32.const 0
  call $~lib/rt/__newArray
  local.tee $5
  i32.load offset=4
  drop
  local.get $5
  i32.const 0
  i32.const 4672
  i32.const 4704
  i32.const 1
  call $assembly/index/GreetingConfig#constructor
  call $~lib/array/Array<assembly/index/GreetingConfig>#__set
  local.get $5
  i32.const 1
  i32.const 4736
  i32.const 4768
  i32.const 2
  call $assembly/index/GreetingConfig#constructor
  call $~lib/array/Array<assembly/index/GreetingConfig>#__set
  local.get $5
  i32.const 2
  i32.const 4800
  i32.const 4848
  i32.const 3
  call $assembly/index/GreetingConfig#constructor
  call $~lib/array/Array<assembly/index/GreetingConfig>#__set
  loop $for-loop|0
   local.get $3
   local.get $5
   i32.load offset=12
   i32.lt_s
   if
    i32.const 0
    local.set $2
    i32.const 0
    local.set $4
    local.get $5
    local.get $3
    call $~lib/array/Array<assembly/index/GreetingConfig>#__get
    local.tee $6
    i32.load offset=8
    local.set $7
    i32.const 2128
    local.set $0
    loop $for-loop|00
     local.get $4
     local.get $7
     i32.lt_s
     if
      local.get $0
      i32.const 4704
      call $~lib/string/String.__concat
      local.set $0
      local.get $4
      i32.const 1
      i32.add
      local.set $4
      br $for-loop|00
     end
    end
    i32.const 4
    i32.const 9
    i32.const 0
    call $~lib/rt/__newArray
    local.tee $4
    i32.load offset=4
    drop
    local.get $4
    i32.const 0
    local.get $6
    i32.load
    call $~lib/array/Array<assembly/index/GreetingConfig>#__set
    local.get $4
    i32.const 1
    i32.const 5056
    call $~lib/array/Array<assembly/index/GreetingConfig>#__set
    local.get $4
    i32.const 2
    local.get $6
    i32.load offset=4
    call $~lib/array/Array<assembly/index/GreetingConfig>#__set
    local.get $4
    i32.const 3
    local.get $0
    call $~lib/array/Array<assembly/index/GreetingConfig>#__set
    i32.const 2128
    local.set $0
    loop $for-loop|001
     local.get $2
     local.get $4
     i32.load offset=12
     i32.lt_s
     if
      local.get $0
      local.get $4
      local.get $2
      call $~lib/array/Array<assembly/index/GreetingConfig>#__get
      call $~lib/string/String.__concat
      local.set $0
      local.get $2
      i32.const 1
      i32.add
      local.set $2
      br $for-loop|001
     end
    end
    i32.const 5120
    local.get $0
    call $~lib/string/String.__concat
    call $assembly/index/logMessage
    local.get $3
    i32.const 1
    i32.add
    local.set $3
    br $for-loop|0
   end
  end
  i32.const 5152
  call $assembly/index/logMessage
  i32.const 4608
  call $assembly/index/logMessage
  i32.const 5216
  i32.const 10
  local.get $8
  local.get $8
  i32.const 10
  i32.ge_s
  select
  local.tee $0
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  i32.const 5264
  call $~lib/string/String.__concat
  local.get $0
  call $assembly/index/factorial
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 5296
  local.get $0
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  i32.const 5264
  call $~lib/string/String.__concat
  local.set $5
  local.get $0
  local.set $3
  local.get $0
  i32.const 1
  i32.gt_s
  if
   i32.const 1
   local.set $0
   i32.const 2
   local.set $4
   loop $for-loop|003
    local.get $3
    local.get $4
    i32.ge_s
    if
     local.get $0
     local.get $1
     i32.add
     local.set $2
     local.get $0
     local.set $1
     local.get $2
     local.set $0
     local.get $4
     i32.const 1
     i32.add
     local.set $4
     br $for-loop|003
    end
   end
  end
  local.get $5
  local.get $0
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 5344
  call $assembly/index/logMessage
  i32.const 5440
  call $assembly/index/logMessage
  i32.const 5536
  i32.const 127
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 5584
  i32.const 255
  call $~lib/util/number/utoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 5632
  i32.const 2147483647
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 5680
  i32.const -1
  call $~lib/util/number/utoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 5728
  f32.const 3.141590118408203
  call $~lib/number/F32#toString
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  f64.const 2.718281828459045
  i32.const 0
  call $~lib/util/number/dtoa_core
  i32.const 1
  i32.shl
  local.tee $0
  i32.const 2
  call $~lib/rt/tcms/__new
  local.tee $1
  i32.const 5920
  local.get $0
  memory.copy
  i32.const 6912
  local.get $1
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 6960
  i32.const 7008
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 7120
  i32.const 5
  i32.const 8
  i32.const 7072
  call $~lib/rt/__newArray
  i32.load offset=12
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 7184
  i32.const 10
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 7264
  call $assembly/index/logMessage
  i32.const 4608
  call $assembly/index/logMessage
  i32.const 7328
  global.get $assembly/index/stats
  i32.load
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 7392
  global.get $assembly/index/stats
  f32.load offset=8
  call $~lib/number/F32#toString
  call $~lib/string/String.__concat
  call $assembly/index/logMessage
  i32.const 7456
  call $assembly/index/logMessage
  i32.const 7536
  call $assembly/index/logMessage
  i32.const 7616
  call $assembly/index/logMessage
  i32.const 7712
  call $assembly/index/logMessage
  i32.const 7792
  call $assembly/index/logMessage
  i32.const 7888
  call $assembly/index/logMessage
  i32.const 7968
  call $assembly/index/logMessage
  i32.const 8048
  call $assembly/index/logMessage
 )
 (func $assembly/index/main
  i32.const 1280
  call $assembly/index/logMessage
  call $assembly/index/analyze
 )
 (func $assembly/index/run
  i32.const 8112
  call $assembly/index/logMessage
  call $assembly/index/analyze
 )
 (func $assembly/index/getHeapSize (result i32)
  memory.size
  i32.const 16
  i32.shl
 )
 (func $assembly/index/getAllocatedMemory (result i32)
  i32.const 8204
 )
 (func $~start
  (local $0 i32)
  i32.const 1236
  i32.const 1232
  i32.store
  i32.const 1240
  i32.const 1232
  i32.store
  i32.const 1232
  global.set $~lib/rt/tcms/fromSpace
  i32.const 12
  i32.const 4
  call $~lib/rt/tcms/__new
  local.tee $0
  i32.eqz
  if
   i32.const 0
   i32.const 0
   call $~lib/rt/tcms/__new
   local.set $0
  end
  local.get $0
  i32.const 0
  i32.store
  local.get $0
  i32.const 0
  i32.store offset=4
  local.get $0
  f32.const 0
  f32.store offset=8
  local.get $0
  global.set $assembly/index/stats
 )
)
