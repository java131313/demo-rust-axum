package com.wubi.controller;

import com.wubi.dto.ProgressRequest;
import com.wubi.entity.UserProgress;
import com.wubi.service.ProgressService;
import jakarta.validation.Valid;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api")
public class ProgressController {
    @Autowired
    private ProgressService progressService;

    @PostMapping("/progress")
    public ResponseEntity<UserProgress> saveProgress(@Valid @RequestBody ProgressRequest request) {
        return ResponseEntity.ok(progressService.saveProgress(request));
    }
}
