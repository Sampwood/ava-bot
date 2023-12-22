import React from 'react'
import ReactDOM from 'react-dom/client'
import { IconProvider, DEFAULT_ICON_CONFIGS } from '@icon-park/react'

import App from './App.tsx'
import './index.css'

const iconConfig = { ...DEFAULT_ICON_CONFIGS, prefix: 'ava' }

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <IconProvider value={iconConfig}>
      <App />
    </IconProvider>
  </React.StrictMode>
)
