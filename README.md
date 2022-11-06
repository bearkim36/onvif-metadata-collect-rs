# dkant onvif metadata collect 
노변 엣지디바이스와 대용량 메타데이터를 처리할 수 있는 RUST 기반의 ONVIF 메타데이터 수집서버

## Edge Device Featuers

*   [x] 한화 AI카메라 Metadata 수집
*   [ ] 트루엔 AI카메라 Metadata 수집
*   [x] 카메라 Error 발생 시 Failover 처리
    *   [x] 랜선 뽑기
    *   [x] 인위적 통신 차단
    *   [x] 장기간 메타데이터 수집 안될 시 Keep-Alive 처리
*   [x] Xml to JSON 처리
*   [x] 멀티쓰레드로 분석서버로 HTTP Request처리 
*   [x] 설치 가이드 라인 


## Collect Server Featuers

*   [x] 한화 AI카메라 Metadata 수집
*   [ ] 트루엔 AI카메라 Metadata 수집
*   [x] 카메라 Error 발생 시 Failover 처리
    *   [x] 랜선 뽑기
    *   [x] 인위적 통신 차단
    *   [x] 장기간 메타데이터 수집 안될 시 Keep-Alive 처리
*   [x] Xml to JSON 처리
*   [x] 공유 데이터 메모리 관리 기능 
*   [x] 차량 번호 판독기 (TS-ANPR) 테스트 완료 
    *   [ ] 실시간 차량사진 수집
    *   [ ] 실시간 차량번호 판독
*   [x] 안면인식 데이터 테스트 완료 
    *   [ ] 실시간 안면인식 데이터 수집 
*   [ ] 분석서버에 데이터 전송 
    *   [ ] 데이터 스키마 표준화 
*   [x] 설치 가이드 라인

## Install guilde
### Windows (Edge Device)
cargo build release 명령을 수행하여 릴리즈 버전으로 빌드한다.

```sh
    cargo build --release
```

./target/release안에 .env파일을 복사 해 넣은 뒤, 파일에서 다음의 항목들을 수정한다.

```env

# USE mode #1 Edge Device , #2 Server
MODE = 1

# Edge device configuration (Edge only)
RTSP_URL = rtsp://192.168.0.7/profile1/media.smp
RTSP_ID = admin
RTSP_PW = r00tr00tr00t

# Analysis Server Information
ANALYSIS_SERVER_URL = http://127.0.0.1:8010/recvMetadata

```
커맨드 쉘 (관리자용)에서 다음과 같이 PM2와  인스톨한다.
```sh
npm install pm2 -g
npm install pm2-windows-startup -g
pm2-startup install

#exe 파일을 pm2로 실행
pm2 start onvif-metadata-rs.exe 

#pm2 상태 저장
pm2 save
```

### 나머지는 이후 추가 예정

