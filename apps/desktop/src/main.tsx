import React from 'react'
import { createRoot } from 'react-dom/client'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import App from './App'
import Dashboard from './pages/Dashboard'
import Laps from './pages/Laps'
import Track from './pages/Track'
import Workspace from './pages/Workspace'
import './index.css'

const router = createBrowserRouter([
  { path: '/', element: <App />, children: [
    { index: true, element: <Dashboard/> },
    { path: 'laps', element: <Laps/> },
    { path: 'track', element: <Track/> },
    { path: 'workspace', element: <Workspace/> },
  ]}
])

createRoot(document.getElementById('root')!).render(<RouterProvider router={router}/>)
