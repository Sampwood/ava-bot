import { useEffect, useState } from 'react'

function App() {
  const [msgList, setMsgList] = useState<string[]>([])

  useEffect(() => {
    const sse = new EventSource('/api/chats')
    sse.onmessage = (e) => {
      setMsgList(list => [...list, e.data])
    }

    sse.onerror = () => sse.close()

    return () => {
      sse.close()
    }
  }, [])

  return (
    <div className="w-2/3 mx-auto items-center justify-center p-2 mt-2">
      <h1 className="text-center text-2xl">Hello world</h1>
      <ul>
        Contents of this box will be updated in real time with every SSE message received from the
        chatroom.

        {
          msgList.map((msg, index) => <li key={index}>{msg}</li>)
        }
      </ul>
    </div>
  )
}

export default App
