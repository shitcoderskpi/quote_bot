SVG message format spec:
```
0;[byte length];[svg data]
```

Pango markup message format spec:
```
1;[byte length];[x];[y];[wrap width];[wrap mode];[alignment];[font_description];[markup],
```

Metadata message format spec:
```
2;[byte length];[chat_id],[other blocks]
```
Alignment:<br>
0 = left <br>
1 = center <br>
2 = end