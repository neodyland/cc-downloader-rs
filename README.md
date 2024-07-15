# 使い方(docker)
1. `docker/paths`に
```
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00000.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00001.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00002.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00003.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00004.warc.gz
```
(warcのパス)を記入します。  
2. `./run.sh`します。  
3. `docker/output/*.jsonl.zstd`に出力されます。

# 使い方(ソースからビルド)
1. `paths`に
```
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00000.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00001.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00002.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00003.warc.gz
crawl-data/CC-MAIN-2024-26/segments/1718198861173.16/warc/CC-MAIN-20240612140424-20240612170424-00004.warc.gz
```
(warcのパス)を記入します。  
2. `cargo run -r`します。  
3. `output/*.jsonl.zstd`に出力されます。