import { useState, useEffect } from 'react'
import './App.css'
import 'tailwindcss/tailwind.css'

const DEV_MODE = false;

interface Philosopher {
  username: string
  ip: string
  port: number
  of_type: string
  state: string
}

interface Cutlery {
  username: string
  ip: string
  port: number
  of_type: string
  state: Record<string, boolean>
}

interface Response {
  phillosophers: Philosopher[]
  cutlery: Cutlery[]
  state_stats: Record<string, number>
}

function App() {
  const [data, setData] = useState<Response | null>(null)
  const [prevData, setPrevData] = useState<Response | null>(null)

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch('/api');
        const parsed_response: Response = await response.json();

        if (!hasChanged(data, parsed_response)) {
          return;
        }

        setPrevData(data);
        setData(parsed_response);
      } catch (_) {
        console.error('Failed to fetch data');
      }
    };

    const intervalId = setInterval(fetchData, 100);

    return () => clearInterval(intervalId);
  }, [data]);

  const hasChanged = (prev: unknown, current: unknown) => {
    return JSON.stringify(prev) !== JSON.stringify(current);
  };

  return (
    <div className="p-4 w-full flex flex-col items-center">
      <h1 className="text-4xl font-bold mb-4">Dining Philosophers</h1>
      <h2 className="text-2xl font-bold mb-4">Stats:</h2>
      <div className="mb-4">
        {data && (
          <div>
            {Object.entries(data.state_stats).map(([key, value]) => {
              const hasChanged = prevData && prevData.state_stats[key] !== value;

              return (
                <div key={key} className={`mb-2 ${hasChanged ? 'bg-yellow-200' : ''}`}>
                  <strong>{key}:</strong> {value}
                </div>
              )})
            }
          </div>
        )}
      </div>

      <h2 className="text-2xl font-bold mb-4">Fetched Data:</h2>
      {data ? (
        <div>
          <div className="mb-4">
            <h3 className="text-xl font-semibold">Philosophers</h3>
            <ul className="list-none pl-5">
              {data.phillosophers.map((philosopher, index) => (
                <li key={index} className={`mb-2 ${prevData && hasChanged(prevData.phillosophers[index], philosopher) ? 'bg-yellow-200' : ''}`}>
                  <div><strong>Username:</strong> {philosopher.username}</div>
                  <div><strong>IP:</strong> {philosopher.ip}</div>
                  <div><strong>Port:</strong> {philosopher.port}</div>
                  <div><strong>Type:</strong> {philosopher.of_type}</div>
                  <div><strong>State:</strong> {philosopher.state}</div>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <h3 className="text-xl font-semibold">Cutlery</h3>
            <ul className="list-none pl-5">
              {data.cutlery.map((item, index) => (
                <li key={index} className={`mb-2  ${prevData && hasChanged(prevData.cutlery[index], item) ? 'bg-yellow-200' : ''}`}>
                  <div><strong>Username:</strong> {item.username}</div>
                  <div><strong>IP:</strong> {item.ip}</div>
                  <div><strong>Port:</strong> {item.port}</div>
                  <div><strong>Type:</strong> {item.of_type}</div>
                  <div><strong>State:</strong> {
                    JSON.stringify(item.state)
                  } </div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      ) : (
        <div>Loading...</div>
      )}
    </div>
  )
}

export default App
