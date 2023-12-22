import { useEffect, useState } from 'react'
import { Voice } from '@icon-park/react'

let chunks: Blob[] = []
let mediaRecorder: MediaRecorder | null = null

function App() {
  const [isSupported, setIsSupported] = useState(false)
  const [isRecording, setIsRecording] = useState(false)
  const [msgList, setMsgList] = useState<string[]>([])

  useEffect(() => {
    checkMedia()

    const sse = new EventSource('/api/chats')
    sse.onmessage = (e) => {
      setMsgList((list) => [...list, e.data])
    }

    sse.onerror = () => sse.close()

    return () => {
      sse.close()
    }
  }, [])

  function checkMedia() {
    if (mediaRecorder) return

    if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
      navigator.mediaDevices.getUserMedia({ audio: true }).then(
        function (stream) {
          mediaRecorder = new MediaRecorder(stream)

          mediaRecorder.ondataavailable = function (e) {
            chunks.push(e.data)
          }

          mediaRecorder.onstop = () => stopRecording(chunks)
          setIsSupported(true)
        },
        function (err) {
          alert('something is wrong!')
          console.log('The following getUserMedia error occured: ' + err)
        }
      )
    } else {
      alert('getUserMedia not supported on your browser!')
    }
  }

  function toggleRecord() {
    if (!mediaRecorder) return

    if (!isRecording) {
      chunks = []
      mediaRecorder?.start()
    } else {
      mediaRecorder?.stop()
    }

    setIsRecording(!isRecording)
  }

  async function stopRecording(chunks: Blob[]) {
    const formData = new FormData()
    formData.append('audio', new Blob(chunks, { type: 'audio/mp3; codecs=opus' }))

    const res = await fetch('/api/assistant', {
      method: 'POST',
      body: formData,
    })
    console.log(res);
  }

  return (
    <div className="w-2/3 mx-auto items-center justify-center p-2 mt-2">
      <h1 className="text-center text-2xl">Hello world</h1>
      <ul>
        Contents of this box will be updated in real time with every SSE message received from the
        chatroom.
        {/* {msgList.map((msg, index) => (
          <li key={index}>{msg}</li>
        ))} */}
      </ul>

      <section className="sound-clips"></section>

      <div className="mt-10 flex justify-center">
        {/* TODO: 开始录音时，增加心跳动画 */}
        <button
          className={`w-12 h-12 rounded-full text-white flex justify-center items-center ${
            isRecording ? 'animate-bounce' : ''
          } ${isSupported ? 'cursor-pointer bg-red-500' : 'cursor-not-allowed bg-gray-400'}`}
          disabled={!isSupported}
          onClick={toggleRecord}
        >
          <Voice className="text-xl" />
        </button>
      </div>
    </div>
  )
}

export default App
