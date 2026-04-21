package com.wubi.service;

import com.wubi.entity.Article;
import com.wubi.repository.ArticleRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import java.util.List;
import java.util.Optional;

@Service
public class ArticleService {
    @Autowired
    private ArticleRepository articleRepository;

    public List<Article> getAllArticles() {
        return articleRepository.findAll();
    }

    public Optional<Article> getArticleById(Long id) {
        return articleRepository.findById(id);
    }

    public Article createArticle(Article article) {
        return articleRepository.save(article);
    }

    public Article updateArticle(Long id, Article article) {
        Optional<Article> existing = articleRepository.findById(id);
        if (existing.isPresent()) {
            Article updated = existing.get();
            updated.setTitle(article.getTitle());
            updated.setContent(article.getContent());
            updated.setDifficulty(article.getDifficulty());
            return articleRepository.save(updated);
        }
        return null;
    }

    public boolean deleteArticle(Long id) {
        if (articleRepository.existsById(id)) {
            articleRepository.deleteById(id);
            return true;
        }
        return false;
    }
}
