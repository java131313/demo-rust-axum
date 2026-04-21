package com.wubi.service;

import com.wubi.dto.ProgressRequest;
import com.wubi.entity.UserProgress;
import com.wubi.repository.UserProgressRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class ProgressService {
    @Autowired
    private UserProgressRepository userProgressRepository;

    public UserProgress saveProgress(ProgressRequest request) {
        UserProgress progress = new UserProgress();
        progress.setUserName(request.getUserName());
        progress.setLessonId(request.getLessonId());
        progress.setAccuracy(request.getAccuracy());
        progress.setScore(request.getScore());
        return userProgressRepository.save(progress);
    }
}
