import { useState, useEffect } from 'react'
import './App.css'
import 'tailwindcss/tailwind.css'

const DEV_MODE = false;

function App() {
  const [data, setData] = useState(null)
  const [prevData, setPrevData] = useState(null)

  useEffect(() => {
    const fetchData = async () => {
      if (DEV_MODE) {
        const fallbackData = JSON.parse(`{"phillosophers":[{"username":"Camus Wittgenstein 485","ip":"philosopher","port":5094,"of_type":"Philosopher","state":"PhilosopherHungry"},{"username":"Foucault Foucault 16","ip":"philosopher","port":4499,"of_type":"Philosopher","state":"PhilosopherEating"},{"username":"Nietzsche Heidegger 422","ip":"philosopher","port":6395,"of_type":"Philosopher","state":"PhilosopherThinking"}],"cutlery":[{"username":"Spork 830","ip":"cutlery","port":3890,"of_type":"Cutlery","state":{"CutleryClean":false}},{"username":"Spork 534","ip":"cutlery","port":8957,"of_type":"Cutlery","state":{"CutleryClean":false}},{"username":"Fork 849","ip":"cutlery","port":3051,"of_type":"Cutlery","state":{"CutleryClean":false}}]}`);
        setPrevData(data);
        setData(fallbackData);
        console.log('Fetched data:', fallbackData);
      } else {
        try {
          const response = await fetch('/api');
          const parsed_response = await response.json();
          setPrevData(data);
          setData(parsed_response);
        } catch (_) {
          console.error('Failed to fetch data');
        }
      }
    };

    const intervalId = setInterval(fetchData, 100);

    return () => clearInterval(intervalId);
  }, [data]);

  const hasChanged = (prev, current) => {
    return JSON.stringify(prev) !== JSON.stringify(current);
  };

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4">Fetched Data:</h2>
      {data ? (
        <div>
          <div className="mb-4">
            <h3 className="text-xl font-semibold">Philosophers</h3>
            <ul className="list-disc pl-5">
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
            <ul className="list-disc pl-5">
              {data.cutlery.map((item, index) => (
                <li key={index} className={`mb-2  ${prevData && hasChanged(prevData.cutlery[index], item) ? 'bg-yellow-200' : ''}`}>
                  <div><strong>Username:</strong> {item.username}</div>
                  <div><strong>IP:</strong> {item.ip}</div>
                  <div><strong>Port:</strong> {item.port}</div>
                  <div><strong>Type:</strong> {item.of_type}</div>
                  <div><strong>State:</strong> {item.state.CutleryClean ? 'Clean' : 'Not Clean'}</div>
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
