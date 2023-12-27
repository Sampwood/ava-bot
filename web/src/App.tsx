import { useEffect, useState } from 'react'
import { LoadingOne, Voice } from '@icon-park/react'
import Ava from './components/chats/ava'
import User from './components/chats/user'

let chunks: Blob[] = []
let mediaRecorder: MediaRecorder | null = null

type Msg =
  | {
      type: 'user'
      message: string
      data_time: string
    }
  | {
      type: 'ava'
      message: string
      url: string
      data_time: string
    }
type SseData =
  | {
      type: 'signal'
      data: {
        status: string
      }
    }
  | {
      type: 'message'
      data: Msg
    }

function App() {
  const [isSupported, setIsSupported] = useState(false)
  const [isRecording, setIsRecording] = useState(false)
  const [msgList, setMsgList] = useState<Msg[]>([])
  const [status, setStatus] = useState('')

  useEffect(() => {
    checkMedia()

    const sse = new EventSource('/api/chats')
    sse.onmessage = (e) => {
      const data: SseData = JSON.parse(e.data)
      if (data.type === 'message') {
        setMsgList((list) => [...list, data.data])
      } else {
        setStatus(data.data.status === 'done' ? '' : data.data.status)
      }
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
    console.log(res)
  }

  return (
    <div className="w-2/3 mx-auto items-center justify-center p-2 mt-2">
      <h1 className="text-center text-2xl">Hello world</h1>
      <p>
        Contents of this box will be updated in real time with every SSE message received from the
        chatroom.
      </p>
      <ul className="mt-3 rounded-lg p-2 relative border-s border-gray-200">
        {msgList.map((msg, index) =>
          msg.type === 'ava' ? (
            <Ava key={index} time={msg.data_time} message={msg.message} url={msg.url} />
          ) : (
            <User key={index} time={msg.data_time} message={msg.message} />
          )
        )}
      </ul>

      <section className="sound-clips"></section>

      <div className="mt-6 flex justify-center">
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
      {status && (
        <div className="mt-2 flex justify-center items-center">
          <LoadingOne className="animate-spin" />
          <span className="ml-2 text-neutral-400">{status}</span>
        </div>
      )}
    </div>
  )
}

export default App
