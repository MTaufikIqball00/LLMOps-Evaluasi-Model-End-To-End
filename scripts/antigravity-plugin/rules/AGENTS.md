# LLMOps Control Guard (Token, Quality & Security Rules)

- **Efisiensi Token**: Baca file besar per rentang baris (`StartLine`/`EndLine`); hindari membaca ulang bagian file yang sudah ada di konteks; saring output perintah terminal yang panjang; jawab ringkas dan langsung ke inti kecuali diminta detail.
- **Kualitas & Anti-Halusinasi**: Jangan mengklaim isi file, simbol, atau hasil tes yang belum dibaca atau dijalankan langsung; selalu verifikasi perubahan kode dengan linter, build, atau test terkait sebelum menyimpulkan selesai.
- **Keamanan**: Jangan menulis API key, token, atau private key asli ke kode sumber, commit, atau log; jangan menjalankan perintah destruktif tanpa persetujuan eksplisit pengguna.
