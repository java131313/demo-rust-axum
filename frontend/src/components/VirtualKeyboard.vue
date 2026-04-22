<template>
  <div class="virtual-keyboard">
    <div class="keyboard-header">
      <h3>键盘指法提示</h3>
      <p v-if="wubiCode && codeIndex < wubiCode.length">
        编码：<span class="highlight-code">{{ wubiCode }}</span>
        按 <span class="highlight-key">{{ wubiCode[codeIndex] }}</span> 键
      </p>
      <p v-else-if="activeKey">
        按 <span class="highlight-key">{{ activeKey }}</span> 键
      </p>
      <p v-else>等待输入...</p>
    </div>
    
    <!-- 编码显示区域 -->
    <div v-if="wubiCode" class="code-display">
      <div class="code-char" :class="{ current: i === codeIndex }" v-for="(char, i) in wubiCode" :key="i">
        {{ char }}
      </div>
    </div>
    
    <div class="keyboard">
      <!-- 日语假名键盘 -->
      <template v-if="keyboardType === 'japanese'">
        <!-- 第一排 -->
        <div class="row">
          <div class="key" :class="getKeyClass('`')" @mouseenter="showRadical('`')" @mouseleave="hideRadical">`</div>
          <div class="key" :class="getKeyClass('1')" @mouseenter="showRadical('1')" @mouseleave="hideRadical">1</div>
          <div class="key" :class="getKeyClass('2')" @mouseenter="showRadical('2')" @mouseleave="hideRadical">2</div>
          <div class="key" :class="getKeyClass('3')" @mouseenter="showRadical('3')" @mouseleave="hideRadical">3</div>
          <div class="key" :class="getKeyClass('4')" @mouseenter="showRadical('4')" @mouseleave="hideRadical">4</div>
          <div class="key" :class="getKeyClass('5')" @mouseenter="showRadical('5')" @mouseleave="hideRadical">5</div>
          <div class="key" :class="getKeyClass('6')" @mouseenter="showRadical('6')" @mouseleave="hideRadical">6</div>
          <div class="key" :class="getKeyClass('7')" @mouseenter="showRadical('7')" @mouseleave="hideRadical">7</div>
          <div class="key" :class="getKeyClass('8')" @mouseenter="showRadical('8')" @mouseleave="hideRadical">8</div>
          <div class="key" :class="getKeyClass('9')" @mouseenter="showRadical('9')" @mouseleave="hideRadical">9</div>
          <div class="key" :class="getKeyClass('0')" @mouseenter="showRadical('0')" @mouseleave="hideRadical">0</div>
          <div class="key" :class="getKeyClass('-')" @mouseenter="showRadical('-')" @mouseleave="hideRadical">-</div>
          <div class="key" :class="getKeyClass('=')" @mouseenter="showRadical('=')" @mouseleave="hideRadical">=</div>
          <div class="key wide" :class="getKeyClass('Backspace')" @mouseenter="showRadical('Backspace')" @mouseleave="hideRadical">←</div>
        </div>
        
        <!-- 第二排 -->
        <div class="row">
          <div class="key wide" :class="getKeyClass('Tab')" @mouseenter="showRadical('Tab')" @mouseleave="hideRadical">Tab</div>
          <div class="key" :class="getKeyClass('q')" @mouseenter="showRadical('q')" @mouseleave="hideRadical">
            <div class="key-char">Q</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['q'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('w')" @mouseenter="showRadical('w')" @mouseleave="hideRadical">
            <div class="key-char">W</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['w'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('e')" @mouseenter="showRadical('e')" @mouseleave="hideRadical">
            <div class="key-char">E</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['e'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('r')" @mouseenter="showRadical('r')" @mouseleave="hideRadical">
            <div class="key-char">R</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['r'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('t')" @mouseenter="showRadical('t')" @mouseleave="hideRadical">
            <div class="key-char">T</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['t'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('y')" @mouseenter="showRadical('y')" @mouseleave="hideRadical">
            <div class="key-char">Y</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['y'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('u')" @mouseenter="showRadical('u')" @mouseleave="hideRadical">
            <div class="key-char">U</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['u'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('i')" @mouseenter="showRadical('i')" @mouseleave="hideRadical">
            <div class="key-char">I</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['i'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('o')" @mouseenter="showRadical('o')" @mouseleave="hideRadical">
            <div class="key-char">O</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['o'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('p')" @mouseenter="showRadical('p')" @mouseleave="hideRadical">
            <div class="key-char">P</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['p'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('[')" @mouseenter="showRadical('[')" @mouseleave="hideRadical">[</div>
          <div class="key" :class="getKeyClass(']')" @mouseenter="showRadical(']')" @mouseleave="hideRadical">]</div>
          <div class="key wide" :class="getKeyClass('\\')" @mouseenter="showRadical('\\')" @mouseleave="hideRadical">\</div>
        </div>
        
        <!-- 第三排 - 主行 -->
        <div class="row">
          <div class="key wider" :class="getKeyClass('CapsLock')" @mouseenter="showRadical('CapsLock')" @mouseleave="hideRadical">Caps</div>
          <div class="key" :class="getKeyClass('a')" @mouseenter="showRadical('a')" @mouseleave="hideRadical">
            <div class="key-char">A</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['a'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('s')" @mouseenter="showRadical('s')" @mouseleave="hideRadical">
            <div class="key-char">S</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['s'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('d')" @mouseenter="showRadical('d')" @mouseleave="hideRadical">
            <div class="key-char">D</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['d'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('f')" @mouseenter="showRadical('f')" @mouseleave="hideRadical">
            <div class="key-char">F</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['f'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('g')" @mouseenter="showRadical('g')" @mouseleave="hideRadical">
            <div class="key-char">G</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['g'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('h')" @mouseenter="showRadical('h')" @mouseleave="hideRadical">
            <div class="key-char">H</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['h'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('j')" @mouseenter="showRadical('j')" @mouseleave="hideRadical">
            <div class="key-char">J</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['j'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('k')" @mouseenter="showRadical('k')" @mouseleave="hideRadical">
            <div class="key-char">K</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['k'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('l')" @mouseenter="showRadical('l')" @mouseleave="hideRadical">
            <div class="key-char">L</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['l'] }}</div>
          </div>
          <div class="key" :class="getKeyClass(';')" @mouseenter="showRadical(';')" @mouseleave="hideRadical">;</div>
          <div class="key" :class="getKeyClass('\'')" @mouseenter="showRadical('\'')" @mouseleave="hideRadical">'</div>
          <div class="key wider" :class="getKeyClass('Enter')" @mouseenter="showRadical('Enter')" @mouseleave="hideRadical">Enter</div>
        </div>
        
        <!-- 第四排 -->
        <div class="row">
          <div class="key extra-wide" :class="getKeyClass('ShiftLeft')" @mouseenter="showRadical('Shift')" @mouseleave="hideRadical">Shift</div>
          <div class="key" :class="getKeyClass('z')" @mouseenter="showRadical('z')" @mouseleave="hideRadical">
            <div class="key-char">Z</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['z'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('x')" @mouseenter="showRadical('x')" @mouseleave="hideRadical">
            <div class="key-char">X</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['x'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('c')" @mouseenter="showRadical('c')" @mouseleave="hideRadical">
            <div class="key-char">C</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['c'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('v')" @mouseenter="showRadical('v')" @mouseleave="hideRadical">
            <div class="key-char">V</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['v'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('b')" @mouseenter="showRadical('b')" @mouseleave="hideRadical">
            <div class="key-char">B</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['b'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('n')" @mouseenter="showRadical('n')" @mouseleave="hideRadical">
            <div class="key-char">N</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['n'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('m')" @mouseenter="showRadical('m')" @mouseleave="hideRadical">
            <div class="key-char">M</div>
            <div class="key-japanese">{{ japaneseKeyboardLayout.hiragana['m'] }}</div>
          </div>
          <div class="key" :class="getKeyClass(',')" @mouseenter="showRadical(',')" @mouseleave="hideRadical">,</div>
          <div class="key" :class="getKeyClass('.')" @mouseenter="showRadical('.')" @mouseleave="hideRadical">.</div>
          <div class="key" :class="getKeyClass('/')" @mouseenter="showRadical('/')" @mouseleave="hideRadical">/</div>
          <div class="key extra-wide" :class="getKeyClass('ShiftRight')" @mouseenter="showRadical('Shift')" @mouseleave="hideRadical">Shift</div>
        </div>
        
        <!-- 第五排 -->
        <div class="row">
          <div class="key extra-wide" :class="getKeyClass('Control')" @mouseenter="showRadical('Ctrl')" @mouseleave="hideRadical">Ctrl</div>
          <div class="key wide" :class="getKeyClass('Alt')" @mouseenter="showRadical('Alt')" @mouseleave="hideRadical">Alt</div>
          <div class="key space" :class="getKeyClass(' ')" @mouseenter="showRadical(' ')" @mouseleave="hideRadical">空格</div>
          <div class="key wide" :class="getKeyClass('Alt')" @mouseenter="showRadical('Alt')" @mouseleave="hideRadical">Alt</div>
          <div class="key extra-wide" :class="getKeyClass('Control')" @mouseenter="showRadical('Ctrl')" @mouseleave="hideRadical">Ctrl</div>
        </div>
      </template>
      
      <!-- 注音键盘 -->
      <template v-else-if="keyboardType === 'bopomofo'">
        <!-- 第一排 -->
        <div class="row">
          <div class="key" :class="getKeyClass('`')" @mouseenter="showRadical('`')" @mouseleave="hideRadical">`</div>
          <div class="key" :class="getKeyClass('1')" @mouseenter="showRadical('1')" @mouseleave="hideRadical">1</div>
          <div class="key" :class="getKeyClass('2')" @mouseenter="showRadical('2')" @mouseleave="hideRadical">2</div>
          <div class="key" :class="getKeyClass('3')" @mouseenter="showRadical('3')" @mouseleave="hideRadical">3</div>
          <div class="key" :class="getKeyClass('4')" @mouseenter="showRadical('4')" @mouseleave="hideRadical">4</div>
          <div class="key" :class="getKeyClass('5')" @mouseenter="showRadical('5')" @mouseleave="hideRadical">5</div>
          <div class="key" :class="getKeyClass('6')" @mouseenter="showRadical('6')" @mouseleave="hideRadical">6</div>
          <div class="key" :class="getKeyClass('7')" @mouseenter="showRadical('7')" @mouseleave="hideRadical">7</div>
          <div class="key" :class="getKeyClass('8')" @mouseenter="showRadical('8')" @mouseleave="hideRadical">8</div>
          <div class="key" :class="getKeyClass('9')" @mouseenter="showRadical('9')" @mouseleave="hideRadical">9</div>
          <div class="key" :class="getKeyClass('0')" @mouseenter="showRadical('0')" @mouseleave="hideRadical">0</div>
          <div class="key" :class="getKeyClass('-')" @mouseenter="showRadical('-')" @mouseleave="hideRadical">-</div>
          <div class="key" :class="getKeyClass('=')" @mouseenter="showRadical('=')" @mouseleave="hideRadical">=</div>
          <div class="key wide" :class="getKeyClass('Backspace')" @mouseenter="showRadical('Backspace')" @mouseleave="hideRadical">←</div>
        </div>
        
        <!-- 第二排 -->
        <div class="row">
          <div class="key wide" :class="getKeyClass('Tab')" @mouseenter="showRadical('Tab')" @mouseleave="hideRadical">Tab</div>
          <div class="key" :class="getKeyClass('q')" @mouseenter="showRadical('q')" @mouseleave="hideRadical">
            <div class="key-char">Q</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['q'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('w')" @mouseenter="showRadical('w')" @mouseleave="hideRadical">
            <div class="key-char">W</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['w'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('e')" @mouseenter="showRadical('e')" @mouseleave="hideRadical">
            <div class="key-char">E</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['e'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('r')" @mouseenter="showRadical('r')" @mouseleave="hideRadical">
            <div class="key-char">R</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['r'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('t')" @mouseenter="showRadical('t')" @mouseleave="hideRadical">
            <div class="key-char">T</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['t'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('y')" @mouseenter="showRadical('y')" @mouseleave="hideRadical">
            <div class="key-char">Y</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['y'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('u')" @mouseenter="showRadical('u')" @mouseleave="hideRadical">
            <div class="key-char">U</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['u'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('i')" @mouseenter="showRadical('i')" @mouseleave="hideRadical">
            <div class="key-char">I</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['i'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('o')" @mouseenter="showRadical('o')" @mouseleave="hideRadical">
            <div class="key-char">O</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['o'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('p')" @mouseenter="showRadical('p')" @mouseleave="hideRadical">
            <div class="key-char">P</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['p'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('[')" @mouseenter="showRadical('[')" @mouseleave="hideRadical">[</div>
          <div class="key" :class="getKeyClass(']')" @mouseenter="showRadical(']')" @mouseleave="hideRadical">]</div>
          <div class="key wide" :class="getKeyClass('\\')" @mouseenter="showRadical('\\')" @mouseleave="hideRadical">\</div>
        </div>
        
        <!-- 第三排 - 主行 -->
        <div class="row">
          <div class="key wider" :class="getKeyClass('CapsLock')" @mouseenter="showRadical('CapsLock')" @mouseleave="hideRadical">Caps</div>
          <div class="key" :class="getKeyClass('a')" @mouseenter="showRadical('a')" @mouseleave="hideRadical">
            <div class="key-char">A</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['a'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('s')" @mouseenter="showRadical('s')" @mouseleave="hideRadical">
            <div class="key-char">S</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['s'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('d')" @mouseenter="showRadical('d')" @mouseleave="hideRadical">
            <div class="key-char">D</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['d'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('f')" @mouseenter="showRadical('f')" @mouseleave="hideRadical">
            <div class="key-char">F</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['f'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('g')" @mouseenter="showRadical('g')" @mouseleave="hideRadical">
            <div class="key-char">G</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['g'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('h')" @mouseenter="showRadical('h')" @mouseleave="hideRadical">
            <div class="key-char">H</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['h'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('j')" @mouseenter="showRadical('j')" @mouseleave="hideRadical">
            <div class="key-char">J</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['j'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('k')" @mouseenter="showRadical('k')" @mouseleave="hideRadical">
            <div class="key-char">K</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['k'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('l')" @mouseenter="showRadical('l')" @mouseleave="hideRadical">
            <div class="key-char">L</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['l'] }}</div>
          </div>
          <div class="key" :class="getKeyClass(';')" @mouseenter="showRadical(';')" @mouseleave="hideRadical">;</div>
          <div class="key" :class="getKeyClass('\'')" @mouseenter="showRadical('\'')" @mouseleave="hideRadical">'</div>
          <div class="key wider" :class="getKeyClass('Enter')" @mouseenter="showRadical('Enter')" @mouseleave="hideRadical">Enter</div>
        </div>
        
        <!-- 第四排 -->
        <div class="row">
          <div class="key extra-wide" :class="getKeyClass('ShiftLeft')" @mouseenter="showRadical('Shift')" @mouseleave="hideRadical">Shift</div>
          <div class="key" :class="getKeyClass('z')" @mouseenter="showRadical('z')" @mouseleave="hideRadical">
            <div class="key-char">Z</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['z'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('x')" @mouseenter="showRadical('x')" @mouseleave="hideRadical">
            <div class="key-char">X</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['x'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('c')" @mouseenter="showRadical('c')" @mouseleave="hideRadical">
            <div class="key-char">C</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['c'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('v')" @mouseenter="showRadical('v')" @mouseleave="hideRadical">
            <div class="key-char">V</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['v'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('b')" @mouseenter="showRadical('b')" @mouseleave="hideRadical">
            <div class="key-char">B</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['b'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('n')" @mouseenter="showRadical('n')" @mouseleave="hideRadical">
            <div class="key-char">N</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['n'] }}</div>
          </div>
          <div class="key" :class="getKeyClass('m')" @mouseenter="showRadical('m')" @mouseleave="hideRadical">
            <div class="key-char">M</div>
            <div class="key-bopomofo">{{ bopomofoKeyboardLayout['m'] }}</div>
          </div>
          <div class="key" :class="getKeyClass(',')" @mouseenter="showRadical(',')" @mouseleave="hideRadical">,</div>
          <div class="key" :class="getKeyClass('.')" @mouseenter="showRadical('.')" @mouseleave="hideRadical">.</div>
          <div class="key" :class="getKeyClass('/')" @mouseenter="showRadical('/')" @mouseleave="hideRadical">/</div>
          <div class="key extra-wide" :class="getKeyClass('ShiftRight')" @mouseenter="showRadical('Shift')" @mouseleave="hideRadical">Shift</div>
        </div>
        
        <!-- 第五排 -->
        <div class="row">
          <div class="key extra-wide" :class="getKeyClass('Control')" @mouseenter="showRadical('Ctrl')" @mouseleave="hideRadical">Ctrl</div>
          <div class="key wide" :class="getKeyClass('Alt')" @mouseenter="showRadical('Alt')" @mouseleave="hideRadical">Alt</div>
          <div class="key space" :class="getKeyClass(' ')" @mouseenter="showRadical(' ')" @mouseleave="hideRadical">空格</div>
          <div class="key wide" :class="getKeyClass('Alt')" @mouseenter="showRadical('Alt')" @mouseleave="hideRadical">Alt</div>
          <div class="key extra-wide" :class="getKeyClass('Control')" @mouseenter="showRadical('Ctrl')" @mouseleave="hideRadical">Ctrl</div>
        </div>
      </template>
      
      <!-- 标准QWERTY键盘 -->
      <template v-else>
        <!-- 第一排 -->
        <div class="row">
          <div class="key" :class="getKeyClass('`')" @mouseenter="showRadical('`')" @mouseleave="hideRadical">`</div>
          <div class="key" :class="getKeyClass('1')" @mouseenter="showRadical('1')" @mouseleave="hideRadical">1</div>
          <div class="key" :class="getKeyClass('2')" @mouseenter="showRadical('2')" @mouseleave="hideRadical">2</div>
          <div class="key" :class="getKeyClass('3')" @mouseenter="showRadical('3')" @mouseleave="hideRadical">3</div>
          <div class="key" :class="getKeyClass('4')" @mouseenter="showRadical('4')" @mouseleave="hideRadical">4</div>
          <div class="key" :class="getKeyClass('5')" @mouseenter="showRadical('5')" @mouseleave="hideRadical">5</div>
          <div class="key" :class="getKeyClass('6')" @mouseenter="showRadical('6')" @mouseleave="hideRadical">6</div>
          <div class="key" :class="getKeyClass('7')" @mouseenter="showRadical('7')" @mouseleave="hideRadical">7</div>
          <div class="key" :class="getKeyClass('8')" @mouseenter="showRadical('8')" @mouseleave="hideRadical">8</div>
          <div class="key" :class="getKeyClass('9')" @mouseenter="showRadical('9')" @mouseleave="hideRadical">9</div>
          <div class="key" :class="getKeyClass('0')" @mouseenter="showRadical('0')" @mouseleave="hideRadical">0</div>
          <div class="key" :class="getKeyClass('-')" @mouseenter="showRadical('-')" @mouseleave="hideRadical">-</div>
          <div class="key" :class="getKeyClass('=')" @mouseenter="showRadical('=')" @mouseleave="hideRadical">=</div>
          <div class="key wide" :class="getKeyClass('Backspace')" @mouseenter="showRadical('Backspace')" @mouseleave="hideRadical">←</div>
        </div>
        
        <!-- 第二排 -->
        <div class="row">
          <div class="key wide" :class="getKeyClass('Tab')" @mouseenter="showRadical('Tab')" @mouseleave="hideRadical">Tab</div>
          <div class="key" :class="getKeyClass('q')" @mouseenter="showRadical('q')" @mouseleave="hideRadical">Q</div>
          <div class="key" :class="getKeyClass('w')" @mouseenter="showRadical('w')" @mouseleave="hideRadical">W</div>
          <div class="key" :class="getKeyClass('e')" @mouseenter="showRadical('e')" @mouseleave="hideRadical">E</div>
          <div class="key" :class="getKeyClass('r')" @mouseenter="showRadical('r')" @mouseleave="hideRadical">R</div>
          <div class="key" :class="getKeyClass('t')" @mouseenter="showRadical('t')" @mouseleave="hideRadical">T</div>
          <div class="key" :class="getKeyClass('y')" @mouseenter="showRadical('y')" @mouseleave="hideRadical">Y</div>
          <div class="key" :class="getKeyClass('u')" @mouseenter="showRadical('u')" @mouseleave="hideRadical">U</div>
          <div class="key" :class="getKeyClass('i')" @mouseenter="showRadical('i')" @mouseleave="hideRadical">I</div>
          <div class="key" :class="getKeyClass('o')" @mouseenter="showRadical('o')" @mouseleave="hideRadical">O</div>
          <div class="key" :class="getKeyClass('p')" @mouseenter="showRadical('p')" @mouseleave="hideRadical">P</div>
          <div class="key" :class="getKeyClass('[')" @mouseenter="showRadical('[')" @mouseleave="hideRadical">[</div>
          <div class="key" :class="getKeyClass(']')" @mouseenter="showRadical(']')" @mouseleave="hideRadical">]</div>
          <div class="key wide" :class="getKeyClass('\\')" @mouseenter="showRadical('\\')" @mouseleave="hideRadical">\</div>
        </div>
        
        <!-- 第三排 - 主行 -->
        <div class="row">
          <div class="key wider" :class="getKeyClass('CapsLock')" @mouseenter="showRadical('CapsLock')" @mouseleave="hideRadical">Caps</div>
          <div class="key" :class="getKeyClass('a')" @mouseenter="showRadical('a')" @mouseleave="hideRadical">A</div>
          <div class="key" :class="getKeyClass('s')" @mouseenter="showRadical('s')" @mouseleave="hideRadical">S</div>
          <div class="key" :class="getKeyClass('d')" @mouseenter="showRadical('d')" @mouseleave="hideRadical">D</div>
          <div class="key" :class="getKeyClass('f')" @mouseenter="showRadical('f')" @mouseleave="hideRadical">F</div>
          <div class="key" :class="getKeyClass('g')" @mouseenter="showRadical('g')" @mouseleave="hideRadical">G</div>
          <div class="key" :class="getKeyClass('h')" @mouseenter="showRadical('h')" @mouseleave="hideRadical">H</div>
          <div class="key" :class="getKeyClass('j')" @mouseenter="showRadical('j')" @mouseleave="hideRadical">J</div>
          <div class="key" :class="getKeyClass('k')" @mouseenter="showRadical('k')" @mouseleave="hideRadical">K</div>
          <div class="key" :class="getKeyClass('l')" @mouseenter="showRadical('l')" @mouseleave="hideRadical">L</div>
          <div class="key" :class="getKeyClass(';')" @mouseenter="showRadical(';')" @mouseleave="hideRadical">;</div>
          <div class="key" :class="getKeyClass('\'')" @mouseenter="showRadical('\'')" @mouseleave="hideRadical">'</div>
          <div class="key wider" :class="getKeyClass('Enter')" @mouseenter="showRadical('Enter')" @mouseleave="hideRadical">Enter</div>
        </div>
        
        <!-- 第四排 -->
        <div class="row">
          <div class="key extra-wide" :class="getKeyClass('ShiftLeft')" @mouseenter="showRadical('Shift')" @mouseleave="hideRadical">Shift</div>
          <div class="key" :class="getKeyClass('z')" @mouseenter="showRadical('z')" @mouseleave="hideRadical">Z</div>
          <div class="key" :class="getKeyClass('x')" @mouseenter="showRadical('x')" @mouseleave="hideRadical">X</div>
          <div class="key" :class="getKeyClass('c')" @mouseenter="showRadical('c')" @mouseleave="hideRadical">C</div>
          <div class="key" :class="getKeyClass('v')" @mouseenter="showRadical('v')" @mouseleave="hideRadical">V</div>
          <div class="key" :class="getKeyClass('b')" @mouseenter="showRadical('b')" @mouseleave="hideRadical">B</div>
          <div class="key" :class="getKeyClass('n')" @mouseenter="showRadical('n')" @mouseleave="hideRadical">N</div>
          <div class="key" :class="getKeyClass('m')" @mouseenter="showRadical('m')" @mouseleave="hideRadical">M</div>
          <div class="key" :class="getKeyClass(',')" @mouseenter="showRadical(',')" @mouseleave="hideRadical">,</div>
          <div class="key" :class="getKeyClass('.')" @mouseenter="showRadical('.')" @mouseleave="hideRadical">.</div>
          <div class="key" :class="getKeyClass('/')" @mouseenter="showRadical('/')" @mouseleave="hideRadical">/</div>
          <div class="key extra-wide" :class="getKeyClass('ShiftRight')" @mouseenter="showRadical('Shift')" @mouseleave="hideRadical">Shift</div>
        </div>
        
        <!-- 第五排 -->
        <div class="row">
          <div class="key extra-wide" :class="getKeyClass('Control')" @mouseenter="showRadical('Ctrl')" @mouseleave="hideRadical">Ctrl</div>
          <div class="key wide" :class="getKeyClass('Alt')" @mouseenter="showRadical('Alt')" @mouseleave="hideRadical">Alt</div>
          <div class="key space" :class="getKeyClass(' ')" @mouseenter="showRadical(' ')" @mouseleave="hideRadical">空格</div>
          <div class="key wide" :class="getKeyClass('Alt')" @mouseenter="showRadical('Alt')" @mouseleave="hideRadical">Alt</div>
          <div class="key extra-wide" :class="getKeyClass('Control')" @mouseenter="showRadical('Ctrl')" @mouseleave="hideRadical">Ctrl</div>
        </div>
      </template>
    </div>
    
    <!-- 字根提示气泡 -->
    <div v-if="hoveredRadical" class="radical-tooltip">
      <div class="tooltip-key">{{ hoveredKey.toUpperCase() }}</div>
      <div class="tooltip-radicals">{{ hoveredRadical.radicals }}</div>
      <div v-if="hoveredRadical.description" class="tooltip-desc">{{ hoveredRadical.description }}</div>
    </div>
    
    <!-- 手指图例 -->
    <div class="finger-legend">
      <div class="legend-item">
        <span class="color-box pinky"></span>
        <span>小指</span>
      </div>
      <div class="legend-item">
        <span class="color-box ring"></span>
        <span>无名指</span>
      </div>
      <div class="legend-item">
        <span class="color-box middle"></span>
        <span>中指</span>
      </div>
      <div class="legend-item">
        <span class="color-box index-left"></span>
        <span>左手食指</span>
      </div>
      <div class="legend-item">
        <span class="color-box index-right"></span>
        <span>右手食指</span>
      </div>
    </div>
  </div>
</template>

<script>
import axios from 'axios';

export default {
  name: 'VirtualKeyboard',
  props: {
    activeKey: {
      type: String,
      default: null
    },
    wubiCode: {
      type: String,
      default: null
    },
    codeIndex: {
      type: Number,
      default: 0
    },
    keyboardType: {
      type: String,
      default: 'qwerty' // qwerty, japanese, bopomofo
    }
  },
  data() {
    return {
      fingerMap: {
        '`': 'pinky', '1': 'pinky', 'Tab': 'pinky', 'q': 'pinky', 'a': 'pinky', 'z': 'pinky',
        'CapsLock': 'pinky', 'ShiftLeft': 'pinky', 'Control': 'pinky',
        '2': 'ring', 'w': 'ring', 's': 'ring', 'x': 'ring',
        '3': 'middle', 'e': 'middle', 'd': 'middle', 'c': 'middle',
        '4': 'index-left', 'r': 'index-left', 'f': 'index-left', 'v': 'index-left',
        '5': 'index-left', 't': 'index-left', 'g': 'index-left', 'b': 'index-left',
        '6': 'index-right', 'y': 'index-right', 'h': 'index-right', 'n': 'index-right',
        '7': 'index-right', 'u': 'index-right', 'j': 'index-right', 'm': 'index-right',
        '8': 'middle', 'i': 'middle', 'k': 'middle', ',': 'middle',
        '9': 'ring', 'o': 'ring', 'l': 'ring', '.': 'ring',
        '0': 'pinky', 'p': 'pinky', ';': 'pinky', '/': 'pinky',
        '-': 'pinky', '=': 'pinky', '[': 'pinky', ']': 'pinky',
        '\\': 'pinky', '\'': 'pinky', 'Backspace': 'pinky', 'Enter': 'pinky',
        'ShiftRight': 'pinky', 'Alt': 'pinky',
        ' ': 'thumb'
      },
      // 日语假名键盘布局
      japaneseKeyboardLayout: {
        // 平假名映射
        hiragana: {
          'q': 'た', 'w': 'て', 'e': 'い', 'r': 'お', 't': 'や', 'y': 'ゆ', 'u': 'よ', 'i': 'わ', 'o': 'ほ', 'p': 'へ',
          'a': 'か', 's': 'ん', 'd': 'な', 'f': 'に', 'g': 'ら', 'h': 'せ', 'j': 'た', 'k': 'そ', 'l': 'ち',
          'z': 'さ', 'x': 'そ', 'c': 'た', 'v': 'て', 'b': 'い', 'n': 'お', 'm': 'や'
        },
        // 片假名映射
        katakana: {
          'q': 'タ', 'w': 'テ', 'e': 'イ', 'r': 'オ', 't': 'ヤ', 'y': 'ユ', 'u': 'ヨ', 'i': 'ワ', 'o': 'ホ', 'p': 'ヘ',
          'a': 'カ', 's': 'ン', 'd': 'ナ', 'f': 'ニ', 'g': 'ラ', 'h': 'セ', 'j': 'タ', 'k': 'ソ', 'l': 'チ',
          'z': 'サ', 'x': 'ソ', 'c': 'タ', 'v': 'テ', 'b': 'イ', 'n': 'オ', 'm': 'ヤ'
        }
      },
      // 注音键盘布局
      bopomofoKeyboardLayout: {
        'q': 'ㄆ', 'w': 'ㄇ', 'e': 'ㄈ', 'r': 'ㄖ', 't': 'ㄊ', 'y': 'ㄋ', 'u': 'ㄌ', 'i': 'ㄧ', 'o': 'ㄛ', 'p': 'ㄟ',
        'a': 'ㄅ', 's': 'ㄉ', 'd': 'ㄍ', 'f': 'ㄐ', 'g': 'ㄔ', 'h': 'ㄗ', 'j': 'ㄘ', 'k': 'ㄙ', 'l': 'ㄚ',
        'z': 'ㄏ', 'x': 'ㄎ', 'c': 'ㄑ', 'v': 'ㄕ', 'b': 'ㄖ', 'n': 'ㄋ', 'm': 'ㄌ'
      },
      keyRadicals: {},
      hoveredKey: null,
      hoveredRadical: null
    }
  },
  computed: {
    effectiveActiveKey() {
      if (this.wubiCode && this.codeIndex < this.wubiCode.length) {
        return this.wubiCode[this.codeIndex];
      }
      return this.activeKey;
    }
  },
  async mounted() {
    try {
      const res = await axios.get('/api/key-radicals');
      if (res.data) {
        const map = {};
        res.data.forEach(item => {
          map[item.key_char] = item;
        });
        this.keyRadicals = map;
      }
    } catch (e) {
      console.error('加载键位字根失败:', e);
    }
  },
  methods: {
    getKeyClass(key) {
      const classes = ['key'];
      const lowerKey = key.toLowerCase();
      const activeKey = this.effectiveActiveKey;
      
      if (activeKey && activeKey.toLowerCase() === lowerKey) {
        classes.push('active');
      }
      
      const finger = this.fingerMap[key] || this.fingerMap[lowerKey];
      if (finger) {
        classes.push(finger);
      }
      
      return classes.join(' ');
    },
    
    showRadical(key) {
      this.hoveredKey = key;
      const radical = this.keyRadicals[key.toLowerCase()] || this.keyRadicals[key];
      if (radical) {
        this.hoveredRadical = radical;
      } else {
        this.hoveredRadical = null;
      }
    },
    
    hideRadical() {
      this.hoveredKey = null;
      this.hoveredRadical = null;
    }
  }
}
</script>

<style scoped>
.virtual-keyboard {
  background: #1e293b;
  border-radius: 12px;
  padding: 20px;
  margin-top: 20px;
  position: relative;
}

.keyboard-header {
  text-align: center;
  margin-bottom: 16px;
  color: white;
}

.keyboard-header h3 {
  margin: 0 0 10px 0;
  font-size: 1.15rem;
}

.keyboard-header p {
  margin: 0;
  font-size: 1rem;
  color: #cbd5e1;
}

.highlight-code {
  color: #fbbf24;
  font-weight: bold;
  font-size: 1.2rem;
  letter-spacing: 0.12em;
}

.highlight-key {
  color: #fbbf24;
  font-weight: bold;
  font-size: 1.2rem;
}

.code-display {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 16px;
}

.code-char {
  width: 40px;
  height: 40px;
  background: #334155;
  border: 1px solid #475569;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #cbd5e1;
  font-size: 1.1rem;
  font-weight: bold;
}

.code-char.current {
  background: #fbbf24;
  border-color: #f59e0b;
  color: #1e293b;
  animation: pulse 1s infinite;
  transform: scale(1.2);
}

.keyboard {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.row {
  display: flex;
  gap: 6px;
  justify-content: center;
}

.key {
  width: 48px;
  height: 48px;
  background: #334155;
  border: 1px solid #475569;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #e2e8f0;
  font-size: 15px;
  font-weight: 600;
  transition: all 0.15s ease;
  position: relative;
  cursor: pointer;
}

.key:hover {
  background: #475569;
  transform: translateY(-2px);
}

.key.wide {
  width: 72px;
}

.key.wider {
  width: 84px;
}

.key.extra-wide {
  width: 96px;
}

.key.space {
  flex: 1;
  max-width: 300px;
}

.key.pinky {
  background: #7c3aed;
  border-color: #8b5cf6;
}

.key.ring {
  background: #2563eb;
  border-color: #3b82f6;
}

.key.middle {
  background: #059669;
  border-color: #10b981;
}

.key.index-left {
  background: #d97706;
  border-color: #f59e0b;
}

.key.index-right {
  background: #dc2626;
  border-color: #ef4444;
}

.key.thumb {
  background: #0891b2;
  border-color: #06b6d4;
}

.key.active {
  background: #fbbf24 !important;
  border-color: #f59e0b !important;
  color: #1e293b !important;
  transform: scale(1.1);
  box-shadow: 0 0 12px rgba(251, 191, 36, 0.5);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { box-shadow: 0 0 12px rgba(251, 191, 36, 0.5); }
  50% { box-shadow: 0 0 20px rgba(251, 191, 36, 0.8); }
}

.radical-tooltip {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background: #0f172a;
  border: 1px solid #475569;
  border-radius: 8px;
  padding: 14px 18px;
  min-width: 200px;
  font-size: 1.05rem;
  line-height: 1.5;
  text-align: center;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  pointer-events: none;
}

.tooltip-key {
  color: #fbbf24;
  font-size: 1rem;
  font-weight: bold;
  margin-bottom: 6px;
}

.tooltip-radicals {
  color: #e2e8f0;
  font-size: 1.15rem;
  font-weight: bold;
  margin-bottom: 6px;
}

.tooltip-desc {
  color: #cbd5e1;
  font-size: 0.95rem;
  line-height: 1.45;
}

.finger-legend {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #475569;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #cbd5e1;
  font-size: 0.95rem;
}

.color-box {
  width: 18px;
  height: 18px;
  border-radius: 4px;
}

.color-box.pinky {
  background: #7c3aed;
}

.color-box.ring {
  background: #2563eb;
}

.color-box.middle {
  background: #059669;
}

.color-box.index-left {
  background: #d97706;
}

.color-box.index-right {
  background: #dc2626;
}

/* 日语键盘样式 */
.key-char {
  font-size: 12px;
  margin-bottom: 2px;
}

.key-japanese {
  font-size: 16px;
  font-weight: bold;
}

/* 注音键盘样式 */
.key-bopomofo {
  font-size: 16px;
  font-weight: bold;
}
</style>