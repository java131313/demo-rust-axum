package com.wubi.repository;

import com.wubi.entity.KeyRadical;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;
import java.util.Optional;

@Repository
public interface KeyRadicalRepository extends JpaRepository<KeyRadical, Long> {
    Optional<KeyRadical> findByKeyChar(String keyChar);
}
