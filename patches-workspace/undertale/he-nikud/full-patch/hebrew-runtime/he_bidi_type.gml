// L=0,R=1,AL=2,EN=3,AN=4,NSM=5,ES=6,ET=7,CS=8,ON=9,WS=10.
var cp = ord(argument0);
if ((cp >= 1425 && cp <= 1469) || cp == 1471 || cp == 1473 || cp == 1474 || cp == 1476 || cp == 1477 || cp == 1479 || (cp >= 768 && cp <= 879)) return 5;
// F000-F7FF: build-time atomic Hebrew letter + nikud glyphs.
if ((cp >= 1488 && cp <= 1524) || (cp >= 61440 && cp <= 63487)) return 1;
if (cp == 8207) return 1;
if (cp == 8206) return 0;
if (cp >= 48 && cp <= 57) return 3;
if (cp >= 1632 && cp <= 1641) return 4;
if ((cp >= 65 && cp <= 90) || (cp >= 97 && cp <= 122) || (cp >= 192 && cp <= 687)) return 0;
if (cp >= 1536 && cp <= 1791) return 2;
if (cp == 32 || cp == 9 || cp == 160) return 10;
if (cp == 43 || cp == 45) return 6;
if (cp == 36 || cp == 37 || cp == 162 || cp == 163 || cp == 165 || cp == 8362 || cp == 8364) return 7;
if (cp == 44 || cp == 46 || cp == 47 || cp == 58) return 8;
return 9;
