interface IProps {
  type: 'speech' | 'image'
  time: string
  message: string
  url: string
}

export default function User({ time, message, url }: IProps) {
  return (
    <li className="mb-10 ms-6">
      <span className="absolute flex items-center justify-center w-6 h-6 bg-blue-100 rounded-full -start-3 ring-8 ring-white dark:ring-gray-900 dark:bg-blue-900">
        <img className="rounded-full shadow-lg" src="/ava.png" alt="Ava" />
      </span>
      <div className="items-center justify-between p-4 bg-white border border-gray-200 rounded-lg shadow-sm sm:flex dark:bg-gray-700 dark:border-gray-600">
        <time className="mb-1 text-xs font-normal text-gray-400 sm:order-last sm:mb-0">{time}</time>
        <div className="text-sm font-normal text-gray-500 dark:text-gray-300">
          <p className="ml-2">{message}</p>
          <audio src={url} controls autoPlay></audio>
        </div>
      </div>
    </li>
  )
}
