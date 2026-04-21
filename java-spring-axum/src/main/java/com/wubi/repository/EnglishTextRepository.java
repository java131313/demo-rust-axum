package com.wubi.repository;

import com.wubi.entity.EnglishText;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;

@Repository
public interface EnglishTextRepository extends JpaRepository<EnglishText, Long> {
}
