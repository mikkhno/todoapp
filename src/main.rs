use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use ron::ser::to_string;
use ron::de::from_str;

const FILE_NAME: &str = "tasks.txt";

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct Task {
    id: usize,             // Унікальний ідентифікатор завдання
    description: String,   // Опис завдання
    completed: bool,       // Стан виконання
}

#[derive(Serialize, Deserialize, Default)]
struct TaskRepository {
    tasks: Vec<Task>,      // Колекція всіх завдань
    next_id: usize,        // Наступний доступний ID
}

impl TaskRepository {
    /// Додає нове завдання
    fn add_task(&mut self, description: String) {
        self.tasks.push(Task {
            id: self.next_id,
            description,
            completed: false,
        });
        self.next_id += 1;
    }

    /// Редагує існуюче завдання за ID
    fn edit_task(&mut self, id: usize, description: String) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.description = description;
        }
    }

    /// Видаляє завдання за ID
    fn delete_task(&mut self, id: usize) {
        self.tasks.retain(|task| task.id != id);
    }

    /// Позначає завдання як виконане
    fn mark_completed(&mut self, id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.completed = true;
        }
    }

    /// Зберігає список завдань у файл
    fn save_to_file(&self) {
        if let Ok(data) = to_string(self) {
            if let Ok(mut file) = File::create(FILE_NAME) {
                file.write_all(data.as_bytes()).unwrap();
            }
        }
    }

    /// Завантажує список завдань з файлу
    fn load_from_file() -> Self {
        if let Ok(mut file) = File::open(FILE_NAME) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(repository) = from_str(&contents) {
                    return repository;
                }
            }
        }
        TaskRepository::default()
    }
}

/// Основна функція
fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Записничок",
        options,
        Box::new(|_cc| Box::new(ToDoApp::new())),
    );
}

/// Клас для управління графічним інтерфейсом
struct ToDoApp {
    manager: TaskRepository,   // Репозиторій із завданнями
    new_description: String,   // Поле для вводу нового завдання
    edit_description: String,  // Поле для редагування завдання
    show_edit_popup: bool,     // Показувати чи приховувати вікно редагування
    edit_id_task: Option<usize>, // ID завдання, яке редагується
}

impl ToDoApp {
    /// Ініціалізує додаток із завантаженням існуючих завдань
    fn new() -> Self {
        Self {
            manager: TaskRepository::load_from_file(),
            new_description: String::new(),
            edit_description: String::new(),
            show_edit_popup: false,
            edit_id_task: None,
        }
    }
}

impl eframe::App for ToDoApp {
    /// Основний цикл оновлення GUI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📝 Записничок v1.0");

            // Введення нового завдання
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if self.new_description.is_empty() {
                        ui.label(egui::RichText::new("Я маю зробити...").italics().weak());
                    }
                    ui.text_edit_singleline(&mut self.new_description);
                    if ui.button("➕ Додати завдання").clicked() && !self.new_description.is_empty() {
                        self.manager.add_task(self.new_description.clone());
                        self.new_description.clear();
                        self.manager.save_to_file();
                    }
                });

            });

            ui.separator();

            // Відображення існуючих завдань
            if self.manager.tasks.is_empty() {
                ui.label("✨ На сьогодні нічого не заплановано. Гуляй сміло!");
            } else {
                for task in self.manager.tasks.clone() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Стиль для виконаних завдань
                            if task.completed {
                                ui.label(egui::RichText::new(&task.description).strikethrough());
                            } else {
                                ui.label(&task.description);
                            }

                            // Кнопка редагування
                            if ui.button("✏ Редагувати").clicked() {
                                self.show_edit_popup = true;
                                self.edit_id_task = Some(task.id);
                                self.edit_description = task.description.clone();
                            }

                            // Кнопка завершення
                            if !task.completed {
                                if ui.button("✅ Виконав").clicked() {
                                    self.manager.mark_completed(task.id);
                                    self.manager.save_to_file();
                                }
                            }

                            // Кнопка видалення
                            if ui.button("❌ Вилучити").clicked() {
                                self.manager.delete_task(task.id);
                                self.manager.save_to_file();
                            }
                        });
                    });
                }
            }

            // Вікно для редагування завдання
            if self.show_edit_popup {
                egui::Window::new("✏ Редагувати завдання")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("Зміни завдання:");
                        ui.text_edit_singleline(&mut self.edit_description);

                        ui.horizontal(|ui| {
                            if ui.button("💾 Зберегти").clicked() {
                                if let Some(task_id) = self.edit_id_task {
                                    self.manager.edit_task(task_id, self.edit_description.clone());
                                    self.manager.save_to_file();
                                }
                                self.show_edit_popup = false;
                            }

                            if ui.button("❌ Відмінити").clicked() {
                                self.show_edit_popup = false;
                            }
                        });
                    });
            }
        });
    }
}
