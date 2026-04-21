package com.wubi.service;

import com.wubi.entity.EnglishText;
import com.wubi.repository.EnglishTextRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import java.util.List;

@Service
public class EnglishTextService {
    @Autowired
    private EnglishTextRepository englishTextRepository;

    public List<EnglishText> getAllEnglishTexts() {
        return englishTextRepository.findAll();
    }
}
