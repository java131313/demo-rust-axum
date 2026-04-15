import { useEffect, useState } from 'react';
import { Lesson, ProgressUpdate, NewLesson } from './types';

function App() {
  const [lessons, setLessons] = useState<Lesson[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [userName, setUserName] = useState('student');
  const [selectedLessonId, setSelectedLessonId] = useState<number>(1);
  const [accuracy, setAccuracy] = useState('95');
  const [score, setScore] = useState('80');
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [newLesson, setNewLesson] = useState<NewLesson>({
    character: '',
    code: '',
    description: '',
  });

  useEffect(() => {
    fetch('/api/lessons')
      .then((response) => {
        if (!response.ok) {
          throw new Error('无法加载课程列表');
        }
        return response.json();
      })
      .then((data) => {
        setLessons(data);
        if (data.length > 0) {
          setSelectedLessonId(data[0].id);
        }
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, []);

  const submitProgress = async () => {
    setStatusMessage(null);
    const payload: ProgressUpdate = {
      user_name: userName,
      lesson_id: selectedLessonId,
      accuracy: Number(accuracy),
      score: Number(score),
    };

    const response = await fetch('/api/progress', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (response.ok) {
      setStatusMessage('进度已保存。');
    } else {
      setStatusMessage('保存进度失败，请稍后重试。');
    }
  };

  const addLesson = async () => {
    if (!newLesson.character || !newLesson.code || !newLesson.description) {
      setStatusMessage('请填写完整的新课程信息。');
      return;
    }

    const response = await fetch('/api/lessons', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newLesson),
    });

    if (!response.ok) {
      setStatusMessage('新增课程失败。');
      return;
    }

    const lesson = await response.json();
    setLessons((prev) => [...prev, lesson]);
    setNewLesson({ character: '', code: '', description: '' });
    setStatusMessage('新增课程成功。');
  };

  return (
    <div className="app-shell">
      <header>
        <h1>五笔打字教程</h1>
        <p>后端接口：`demo-rust-axum`，数据库名：`wubi`。</p>
      </header>

      <section className="card">
        <h2>课程列表</h2>
        {loading ? (
          <p>加载中……</p>
        ) : error ? (
          <p className="error">{error}</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>字</th>
                <th>编码</th>
                <th>说明</th>
              </tr>
            </thead>
            <tbody>
              {lessons.map((lesson) => (
                <tr key={lesson.id}>
                  <td>{lesson.id}</td>
                  <td>{lesson.character}</td>
                  <td>{lesson.code}</td>
                  <td>{lesson.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section className="card">
        <h2>保存练习进度</h2>
        <div className="form-grid">
          <label>
            用户名
            <input value={userName} onChange={(event) => setUserName(event.target.value)} />
          </label>
          <label>
            课程 ID
            <select value={selectedLessonId} onChange={(event) => setSelectedLessonId(Number(event.target.value))}>
              {lessons.map((lesson) => (
                <option key={lesson.id} value={lesson.id}>
                  {lesson.id} - {lesson.character}
                </option>
              ))}
            </select>
          </label>
          <label>
            正确率
            <input type="number" value={accuracy} onChange={(event) => setAccuracy(event.target.value)} />
          </label>
          <label>
            得分
            <input type="number" value={score} onChange={(event) => setScore(event.target.value)} />
          </label>
        </div>
        <button onClick={submitProgress}>保存进度</button>
      </section>

      <section className="card">
        <h2>新增课程</h2>
        <div className="form-grid">
          <label>
            字
            <input
              value={newLesson.character}
              onChange={(event) => setNewLesson({ ...newLesson, character: event.target.value })}
            />
          </label>
          <label>
            编码
            <input value={newLesson.code} onChange={(event) => setNewLesson({ ...newLesson, code: event.target.value })} />
          </label>
          <label className="full-width">
            说明
            <input
              value={newLesson.description}
              onChange={(event) => setNewLesson({ ...newLesson, description: event.target.value })}
            />
          </label>
        </div>
        <button onClick={addLesson}>新增课程</button>
      </section>

      {statusMessage ? <p className="status">{statusMessage}</p> : null}
    </div>
  );
}

export default App;
