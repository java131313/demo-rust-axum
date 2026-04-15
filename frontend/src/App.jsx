import { useEffect, useState } from 'react';

function App() {
  const [lessons, setLessons] = useState([]);
  const [articles, setArticles] = useState([]);
  const [wubiRoots, setWubiRoots] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [activeTab, setActiveTab] = useState('lessons');
  const [userName, setUserName] = useState('student');
  const [selectedLessonId, setSelectedLessonId] = useState(1);
  const [accuracy, setAccuracy] = useState('95');
  const [score, setScore] = useState('80');
  const [statusMessage, setStatusMessage] = useState(null);
  const [newLesson, setNewLesson] = useState({
    character: '',
    code: '',
    description: '',
  });
  const [newArticle, setNewArticle] = useState({
    title: '',
    content: '',
    difficulty: 'medium',
  });
  const [newWubiRoot, setNewWubiRoot] = useState({
    character: '',
    code: '',
    position: '',
    description: '',
  });
  const [selectedArticleId, setSelectedArticleId] = useState(0);
  const [currentArticle, setCurrentArticle] = useState(null);
  const [userInput, setUserInput] = useState('');
  const [startTime, setStartTime] = useState(null);
  const [elapsedTime, setElapsedTime] = useState(0);
  const [searchCharacter, setSearchCharacter] = useState('');
  const [searchResult, setSearchResult] = useState(null);

  // Load data on component mount
  useEffect(() => {
    Promise.all([
      fetch('/api/lessons').then(res => res.json()),
      fetch('/api/articles').then(res => res.json()),
      fetch('/api/wubi-roots').then(res => res.json())
    ])
    .then(([lessonsData, articlesData, rootsData]) => {
      setLessons(lessonsData);
      setArticles(articlesData);
      setWubiRoots(rootsData);
      if (lessonsData.length > 0) {
        setSelectedLessonId(lessonsData[0].id);
      }
      if (articlesData.length > 0) {
        setSelectedArticleId(articlesData[0].id);
        setCurrentArticle(articlesData[0]);
      }
    })
    .catch((err) => setError(err.message))
    .finally(() => setLoading(false));
  }, []);

  // Timer effect for typing practice
  useEffect(() => {
    let interval = null;
    
    if (startTime !== null) {
      interval = window.setInterval(() => {
        setElapsedTime(Date.now() - startTime);
      }, 1000);
    }
    
    return () => {
      if (interval) window.clearInterval(interval);
    };
  }, [startTime]);

  const submitProgress = async () => {
    setStatusMessage(null);
    const payload = {
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

  const addArticle = async () => {
    if (!newArticle.title || !newArticle.content) {
      setStatusMessage('请填写完整的文章信息。');
      return;
    }

    const response = await fetch('/api/articles', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newArticle),
    });

    if (!response.ok) {
      setStatusMessage('新增文章失败。');
      return;
    }

    const article = await response.json();
    setArticles((prev) => [...prev, article]);
    setNewArticle({ title: '', content: '', difficulty: 'medium' });
    setStatusMessage('新增文章成功。');
  };

  const addWubiRoot = async () => {
    if (!newWubiRoot.character || !newWubiRoot.code || !newWubiRoot.position) {
      setStatusMessage('请填写完整的字根信息。');
      return;
    }

    const response = await fetch('/api/wubi-roots', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newWubiRoot),
    });

    if (!response.ok) {
      setStatusMessage('新增字根失败。');
      return;
    }

    const root = await response.json();
    setWubiRoots((prev) => [...prev, root]);
    setNewWubiRoot({ character: '', code: '', position: '', description: '' });
    setStatusMessage('新增字根成功。');
  };

  const handleArticleChange = (id) => {
    setSelectedArticleId(id);
    const article = articles.find(a => a.id === id) || null;
    setCurrentArticle(article);
    setUserInput('');
    setStartTime(null);
    setElapsedTime(0);
  };

  const startTypingPractice = () => {
    setStartTime(Date.now());
  };

  const resetTypingPractice = () => {
    setUserInput('');
    setStartTime(null);
    setElapsedTime(0);
  };

  const calculateAccuracy = () => {
    if (!currentArticle) return 0;
    const originalText = currentArticle.content.replace(/\s+/g, '');
    const typedText = userInput.replace(/\s+/g, '');
    
    let correctChars = 0;
    for (let i = 0; i < Math.min(originalText.length, typedText.length); i++) {
      if (originalText[i] === typedText[i]) {
        correctChars++;
      }
    }
    
    return originalText.length > 0 ? (correctChars / originalText.length) * 100 : 0;
  };

  const calculateSpeed = () => {
    if (!startTime || elapsedTime === 0 || !currentArticle) return 0;
    const minutes = elapsedTime / 60000; // Convert ms to minutes
    const charsTyped = userInput.length;
    return charsTyped / minutes;
  };

  const searchWubiRoot = async () => {
    if (!searchCharacter.trim()) {
      setSearchResult(null);
      return;
    }

    try {
      const response = await fetch(`/api/search-wubi-root/${encodeURIComponent(searchCharacter)}`);
      if (response.ok) {
        const data = await response.json();
        setSearchResult(data);
      } else {
        setSearchResult(null);
      }
    } catch (error) {
      console.error('搜索字根时出错:', error);
      setSearchResult(null);
    }
  };

  const formatTime = (ms) => {
    const seconds = Math.floor(ms / 1000);
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="app-shell">
      <header>
        <h1>五笔打字教程</h1>
        <p>后端接口：`demo-rust-axum`，数据库名：`wubi`。</p>
        
        <nav className="tabs">
          <button 
            className={activeTab === 'lessons' ? 'active' : ''} 
            onClick={() => setActiveTab('lessons')}
          >
            课程列表
          </button>
          <button 
            className={activeTab === 'practice' ? 'active' : ''} 
            onClick={() => setActiveTab('practice')}
          >
            打字练习
          </button>
          <button 
            className={activeTab === 'roots' ? 'active' : ''} 
            onClick={() => setActiveTab('roots')}
          >
            字根表
          </button>
          <button 
            className={activeTab === 'progress' ? 'active' : ''} 
            onClick={() => setActiveTab('progress')}
          >
            进度管理
          </button>
        </nav>
      </header>

      {/* 课程列表标签页 */}
      {activeTab === 'lessons' && (
        <div>
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
        </div>
      )}

      {/* 打字练习标签页 */}
      {activeTab === 'practice' && (
        <div>
          <section className="card">
            <h2>选择练习文章</h2>
            <div className="form-grid">
              <label>
                选择文章
                <select 
                  value={selectedArticleId} 
                  onChange={(e) => handleArticleChange(Number(e.target.value))}
                >
                  {articles.map((article) => (
                    <option key={article.id} value={article.id}>
                      {article.title} ({article.difficulty})
                    </option>
                  ))}
                </select>
              </label>
            </div>
          </section>

          {currentArticle && (
            <section className="card">
              <h2>五笔打字练习 - {currentArticle.title}</h2>
              
              <div className="typing-stats">
                <div>时间: {formatTime(elapsedTime)}</div>
                <div>准确率: {calculateAccuracy().toFixed(2)}%</div>
                <div>速度: {calculateSpeed().toFixed(2)} 字/分钟</div>
              </div>
              
              <div className="typing-area">
                <div className="original-text">
                  {currentArticle.content.split('').map((char, index) => {
                    let className = 'char';
                    if (index < userInput.length) {
                      className += userInput[index] === char ? ' correct' : ' incorrect';
                    } else if (index === userInput.length) {
                      className += ' current';
                    }
                    return (
                      <span key={index} className={className}>
                        {char}
                      </span>
                    );
                  })}
                </div>
                
                <textarea
                  className="user-input"
                  value={userInput}
                  onChange={(e) => {
                    if (startTime === null) {
                      startTypingPractice();
                    }
                    setUserInput(e.target.value);
                  }}
                  placeholder="在这里开始输入..."
                  disabled={currentArticle.content.length === 0}
                />
              </div>
              
              <div className="typing-controls">
                <button onClick={resetTypingPractice}>重新开始</button>
              </div>
            </section>
          )}

          <section className="card">
            <h2>新增练习文章</h2>
            <div className="form-grid">
              <label>
                标题
                <input
                  value={newArticle.title}
                  onChange={(event) => setNewArticle({ ...newArticle, title: event.target.value })}
                />
              </label>
              <label>
                难度
                <select
                  value={newArticle.difficulty}
                  onChange={(event) => setNewArticle({ ...newArticle, difficulty: event.target.value })}
                >
                  <option value="easy">简单</option>
                  <option value="medium">中等</option>
                  <option value="hard">困难</option>
                </select>
              </label>
              <label className="full-width">
                内容
                <textarea
                  value={newArticle.content}
                  onChange={(event) => setNewArticle({ ...newArticle, content: event.target.value })}
                  rows={4}
                />
              </label>
            </div>
            <button onClick={addArticle}>新增文章</button>
          </section>
        </div>
      )}

      {/* 字根表标签页 */}
      {activeTab === 'roots' && (
        <div>
          <section className="card">
            <h2>五笔字根表</h2>
            
            <div className="search-box">
              <input
                type="text"
                value={searchCharacter}
                onChange={(e) => setSearchCharacter(e.target.value)}
                placeholder="输入汉字查询五笔编码"
              />
              <button onClick={searchWubiRoot}>查询</button>
            </div>
            
            {searchResult && (
              <div className="search-result">
                <h3>查询结果</h3>
                <p>字符: <strong>{searchResult.character}</strong></p>
                <p>编码: <strong>{searchResult.code}</strong></p>
                <p>位置: {searchResult.position}</p>
                <p>描述: {searchResult.description}</p>
              </div>
            )}
            
            <div className="wubi-grid">
              {wubiRoots.map((root) => (
                <div key={root.id} className="wubi-cell">
                  <div className="wubi-char">{root.character}</div>
                  <div className="wubi-code">{root.code}</div>
                  <div className="wubi-position">{root.position}</div>
                  <div className="wubi-desc">{root.description}</div>
                </div>
              ))}
            </div>
          </section>

          <section className="card">
            <h2>新增字根</h2>
            <div className="form-grid">
              <label>
                字符
                <input
                  value={newWubiRoot.character}
                  onChange={(event) => setNewWubiRoot({ ...newWubiRoot, character: event.target.value })}
                />
              </label>
              <label>
                编码
                <input
                  value={newWubiRoot.code}
                  onChange={(event) => setNewWubiRoot({ ...newWubiRoot, code: event.target.value })}
                />
              </label>
              <label>
                位置
                <input
                  value={newWubiRoot.position}
                  onChange={(event) => setNewWubiRoot({ ...newWubiRoot, position: event.target.value })}
                />
              </label>
              <label className="full-width">
                描述
                <input
                  value={newWubiRoot.description}
                  onChange={(event) => setNewWubiRoot({ ...newWubiRoot, description: event.target.value })}
                />
              </label>
            </div>
            <button onClick={addWubiRoot}>新增字根</button>
          </section>
        </div>
      )}

      {/* 进度管理标签页 */}
      {activeTab === 'progress' && (
        <div>
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
        </div>
      )}

      {statusMessage ? <p className="status">{statusMessage}</p> : null}
    </div>
  );
}

export default App;